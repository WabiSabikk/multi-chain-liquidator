//! HyperEVM alt factory routing — builds swap calldata for the alt UniswapV3 router
//! which has 10-100x more liquidity than the default HyperSwap factory.
//!
//! Routes (collateral, debt) pairs to optimal swap paths. Multi-hop goes through wHYPE.

use alloy::primitives::{address, Address, Bytes, U256};
use alloy::sol;
use alloy::sol_types::SolCall;
use tracing::{info, warn};

/// Alt UniswapV3 router on HyperEVM (better liquidity than default HyperSwap)
pub const ALT_ROUTER: Address = address!("1EBDfC75fFE3bA3De61e7138a3e8706AC841Af9B");

// Token addresses on HyperEVM
const WHYPE: Address = address!("5555555555555555555555555555555555555555");
const KHYPE: Address = address!("fd739d4e423301ce9385c1fb8850539d657c296d");
const WSTHYPE: Address = address!("94e8396e0869c9F2200760aF0621aFd240E1CF38");
const BEHYPE: Address = address!("d8FC8F0b03eBA61F64D08B0bef69d80916E5DdA9");
const UETH: Address = address!("Be6727B535545C67d5cAa73dEa54865B92CF7907");
const UBTC: Address = address!("9FDBdA0A5e284c32744D2f17Ee5c74B284993463");
const USDC: Address = address!("b88339CB7199b77E23DB6E890353E22632Ba630f");
const USDT0: Address = address!("B8CE59FC3717ada4C02eaDF9682A9e934F625ebb");
const USDE: Address = address!("5d3a1Ff2b6BAb83b63cd9AD0787074081a52ef34");

// Uniswap V3 SwapRouter exactInput interface
sol! {
    struct ExactInputParams {
        bytes path;
        address recipient;
        uint256 deadline;
        uint256 amountIn;
        uint256 amountOutMinimum;
    }

    function exactInput(ExactInputParams calldata params) external payable returns (uint256 amountOut);
}

struct Route {
    tokens: Vec<Address>,
    fees: Vec<u32>,
}

/// Build swap calldata for a collateral→debt swap via the alt HyperEVM router.
/// Returns `(calldata, router_address)` for use with `executeLiquidation_1`.
///
/// `amount_in`: estimated collateral amount (raw token units) to swap.
/// `recipient`: the contract address that will receive the output tokens.
pub fn build_swap_calldata(
    collateral: Address,
    debt: Address,
    amount_in: U256,
    recipient: Address,
) -> Option<(Bytes, Address)> {
    if collateral == debt {
        return None;
    }

    let route = match find_route(collateral, debt) {
        Some(r) => r,
        None => {
            warn!(
                from = %collateral,
                to = %debt,
                "HyperSwap: no route found for this pair"
            );
            return None;
        }
    };
    let path = encode_path(&route.tokens, &route.fees);

    let call = exactInputCall {
        params: ExactInputParams {
            path: Bytes::from(path),
            recipient,
            deadline: U256::MAX,
            amountIn: amount_in,
            amountOutMinimum: U256::ZERO,
        },
    };

    let calldata = Bytes::from(call.abi_encode());

    info!(
        from = %collateral,
        to = %debt,
        hops = route.tokens.len() - 1,
        calldata_len = calldata.len(),
        "HyperSwap alt route built"
    );

    Some((calldata, ALT_ROUTER))
}

/// Lookup the routing table for a collateral→debt swap.
fn find_route(from: Address, to: Address) -> Option<Route> {
    if let Some(route) = find_direct(from, to) {
        return Some(route);
    }
    find_multihop_via_whype(from, to)
}

/// Find a direct (single-hop) route between two tokens.
fn find_direct(from: Address, to: Address) -> Option<Route> {
    let fee = match (from, to) {
        // HYPE ecosystem pairs
        (a, b) if (a == KHYPE && b == WHYPE) || (a == WHYPE && b == KHYPE) => 100,
        (a, b) if (a == WSTHYPE && b == WHYPE) || (a == WHYPE && b == WSTHYPE) => 100,
        (a, b) if (a == BEHYPE && b == WHYPE) || (a == WHYPE && b == BEHYPE) => 100,
        (a, b) if (a == UETH && b == WHYPE) || (a == WHYPE && b == UETH) => 3000,

        // wHYPE <-> stables
        (a, b) if (a == WHYPE && b == USDC) || (a == USDC && b == WHYPE) => 500,
        (a, b) if (a == WHYPE && b == USDT0) || (a == USDT0 && b == WHYPE) => 500,
        (a, b) if (a == WHYPE && b == USDE) || (a == USDE && b == WHYPE) => 3000,
        (a, b) if (a == WHYPE && b == UBTC) || (a == UBTC && b == WHYPE) => 3000,

        // Stable <-> stable
        (a, b) if (a == USDC && b == USDT0) || (a == USDT0 && b == USDC) => 100,
        (a, b) if (a == USDE && b == USDC) || (a == USDC && b == USDE) => 100,
        (a, b) if (a == USDE && b == USDT0) || (a == USDT0 && b == USDE) => 100,

        _ => return None,
    };

    Some(Route {
        tokens: vec![from, to],
        fees: vec![fee],
    })
}

/// Find multi-hop route through wHYPE as intermediate.
fn find_multihop_via_whype(from: Address, to: Address) -> Option<Route> {
    if from == WHYPE || to == WHYPE {
        return None;
    }

    let fee_in = fee_to_whype(from)?;
    let fee_out = fee_to_whype(to)?;

    Some(Route {
        tokens: vec![from, WHYPE, to],
        fees: vec![fee_in, fee_out],
    })
}

/// Get the fee tier for a token <-> wHYPE pair on the alt factory.
fn fee_to_whype(token: Address) -> Option<u32> {
    match token {
        t if t == KHYPE => Some(100),
        t if t == WSTHYPE => Some(100),
        t if t == BEHYPE => Some(100),
        t if t == UETH => Some(3000),
        t if t == UBTC => Some(3000),
        t if t == USDC => Some(500),
        t if t == USDT0 => Some(500),
        t if t == USDE => Some(3000),
        _ => None,
    }
}

/// Encode packed path for Uniswap V3:
/// token(20 bytes) + fee(3 bytes) + token(20 bytes) [+ fee(3) + token(20)]...
fn encode_path(tokens: &[Address], fees: &[u32]) -> Vec<u8> {
    debug_assert_eq!(tokens.len(), fees.len() + 1);
    let mut path = Vec::with_capacity(tokens.len() * 20 + fees.len() * 3);
    for i in 0..tokens.len() {
        path.extend_from_slice(tokens[i].as_slice());
        if i < fees.len() {
            // uint24 big-endian = last 3 bytes of u32
            let fee_bytes = fees[i].to_be_bytes();
            path.extend_from_slice(&fee_bytes[1..]);
        }
    }
    path
}
