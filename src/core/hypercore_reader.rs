#![allow(dead_code)]
//! HyperCore precompile reader — reads oracle prices from L1 validator consensus
//! via EVM precompiles at 0x0800+.
//!
//! Precompile 0x0807: Read spot oracle price for a given asset index.
//! HyperCore validators compute weighted median from CEX feeds (Binance w=3, OKX w=2, Bybit w=2).
//! Updates every ~3 seconds.
//!
//! Purpose: measure drift between precompile prices and HyperLend on-chain oracle.
//! If drift > 0 consistently, precompile gives us early signal before oracle updates.

use alloy::primitives::{address, Address, Bytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use tracing::{debug, warn};

/// HyperCore System Oracle precompile address
pub const ORACLE_PRECOMPILE: Address = address!("0000000000000000000000000000000000000807");

/// Known HyperCore asset indices (from HyperLiquid API).
/// These are the perp market indices, NOT token addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum HyperCoreAsset {
    BTC = 0,
    ETH = 1,
    // HYPE index needs to be discovered — try common indices
    // The system_oracle precompile uses internal asset IDs
    HYPE = 131, // Placeholder — will be calibrated at runtime
}

/// Result of a precompile oracle read
#[derive(Debug, Clone)]
pub struct PrecompilePrice {
    pub asset: HyperCoreAsset,
    pub price_raw: U256,
    /// Price in USD (converted from fixed-point)
    pub price_usd: f64,
    pub block: u64,
}

/// Oracle drift measurement: precompile vs HyperLend oracle
#[derive(Debug, Clone)]
pub struct OracleDrift {
    pub block: u64,
    pub asset_symbol: String,
    pub precompile_price: f64,
    pub oracle_price: f64,
    /// Positive = precompile leads (newer price)
    pub drift_pct: f64,
    pub timestamp: u64,
}

/// Read oracle price from HyperCore precompile 0x0807.
///
/// The precompile accepts a uint32 asset index as input and returns
/// the oracle price as a uint256 (fixed-point with 8 decimals).
///
/// Returns None if the precompile call fails (may not be available on all RPCs).
pub async fn read_oracle_price<P: Provider>(
    provider: &P,
    asset_index: u32,
) -> Option<PrecompilePrice> {
    // Encode input: uint32 asset index (left-padded to 32 bytes)
    let mut input = [0u8; 32];
    input[28..32].copy_from_slice(&asset_index.to_be_bytes());

    let tx = TransactionRequest::default()
        .to(ORACLE_PRECOMPILE)
        .input(Bytes::from(input.to_vec()).into());

    match provider.call(tx).await {
        Ok(result) => {
            if result.len() < 32 {
                debug!(asset_index, result_len = result.len(), "Precompile returned short response");
                return None;
            }
            let price_raw = U256::from_be_slice(&result[..32]);
            // HyperCore prices use 8 decimal precision (like Chainlink)
            let price_usd = price_raw.to_string().parse::<f64>().unwrap_or(0.0) / 1e8;

            Some(PrecompilePrice {
                asset: match asset_index {
                    0 => HyperCoreAsset::BTC,
                    1 => HyperCoreAsset::ETH,
                    _ => HyperCoreAsset::HYPE,
                },
                price_raw,
                price_usd,
                block: 0, // caller sets this
            })
        }
        Err(e) => {
            debug!(asset_index, error = %e, "Precompile oracle read failed");
            None
        }
    }
}

/// Measure oracle drift between precompile and HyperLend oracle for a given block.
/// Returns drift percentage (positive = precompile price is higher/newer).
pub fn compute_drift(precompile_price: f64, oracle_price: f64) -> f64 {
    if oracle_price == 0.0 {
        return 0.0;
    }
    (precompile_price - oracle_price) / oracle_price * 100.0
}

/// Try multiple asset indices to find the correct HYPE index.
/// HyperCore assigns sequential indices to perp markets.
/// We try a range and look for a price that matches roughly the known HYPE price.
pub async fn discover_hype_index<P: Provider>(
    provider: &P,
    expected_price_range: (f64, f64), // e.g. (10.0, 50.0) for HYPE
) -> Option<u32> {
    // Try indices 0-200 (HyperLiquid has ~150+ markets)
    for index in 0..200u32 {
        if let Some(price) = read_oracle_price(provider, index).await {
            if price.price_usd >= expected_price_range.0 && price.price_usd <= expected_price_range.1 {
                debug!(index, price = price.price_usd, "Found potential HYPE index");
                return Some(index);
            }
        }
    }
    warn!("Could not discover HYPE index in range {:?}", expected_price_range);
    None
}
