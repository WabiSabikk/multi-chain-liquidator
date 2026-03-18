use crate::core::config::{AavePoolConfig, ChainConfig, TokenInfo, MULTICALL3_ADDRESS};
use alloy::primitives::{address, Address};
use std::collections::HashMap;

fn token(addr: Address, symbol: &str, decimals: u8) -> (Address, TokenInfo) {
    (
        addr,
        TokenInfo {
            address: addr,
            symbol: symbol.to_string(),
            decimals,
        },
    )
}

pub fn chain_config() -> ChainConfig {
    let rpc_url = std::env::var("INK_RPC_URL")
        .unwrap_or_else(|_| "https://ink.drpc.org".to_string());

    let liquidator = std::env::var("INK_LIQUIDATOR_CONTRACT")
        .ok()
        .and_then(|s| s.parse::<Address>().ok())
        .or(Some(address!("D47223b7a191643ecdddC4715C948D88D5a13Bdd")));

    let tokens: HashMap<Address, TokenInfo> = HashMap::from([
        token(address!("4200000000000000000000000000000000000006"), "WETH", 18),
        token(address!("73e0c0d45e048d25fc26fa3159b0aa04bfa4db98"), "kBTC", 8),
        token(address!("0200c29006150606b650577bbe7b6248f58470c1"), "USDT0", 6),
        token(address!("e343167631d89b6ffc58b88d6b7fb0228795491d"), "USDG", 18),
        token(address!("fc421ad3c883bf9e7c4f42de845c4e4405799e73"), "GHO", 18),
        token(address!("2d270e6886d130d724215a266106e6832161eaed"), "USDC", 6),
        token(address!("a3d68b74bf0528fdd07263c60d6488749044914b"), "weETH", 18),
        token(address!("9f0a74a92287e323eb95c1cd9ecdbeb0e397cae4"), "wrsETH", 18),
        token(address!("2416092f143378750bb29b79ed961ab195cceea5"), "ezETH", 18),
        token(address!("211cc4dd073734da055fbf44a2b4667d5e5fe5d2"), "sUSDe", 18),
        token(address!("5d3a1ff2b6bab83b63cd9ad0787074081a52ef34"), "USDe", 18),
        token(address!("ae4efbc7736f963982aacb17efa37fcbab924cb3"), "SolvBTC", 8),
    ]);

    let tydro = AavePoolConfig {
        label: "Tydro".to_string(),
        pool: address!("2816cf15f6d2a220e789aa011d5ee4eb6c47feba"),
        data_provider: address!("96086C25d13943C80Ff9a19791a40Df6aFc08328"),
        oracle: address!("4758213271BFdC72224A7a8742dC865fC97756e1"),
        liquidator_contract: liquidator,
        borrow_start_block: 28_000_000,
        is_v2: false,
        base_currency_decimals: 8,
        cross_token_map: HashMap::new(),
    };

    let wss_url = rpc_url.replace("https://", "wss://");

    ChainConfig {
        chain_id: 57073,
        name: "Ink".to_string(),
        rpc_url,
        wss_url: Some(wss_url),
        enabled: true,
        aave_pools: vec![tydro.clone()],
        aave_pool: tydro.pool,
        aave_data_provider: tydro.data_provider,
        aave_oracle: tydro.oracle,
        liquidator_contract: tydro.liquidator_contract,
        is_v2: false,
        multicall3: MULTICALL3_ADDRESS,
        borrow_start_block: 28_000_000,
        get_logs_chunk_size: 10_000,
        scan_interval_ms: 30_000,
        poll_interval_ms: 3_000,
        tokens,
        min_debt_usd: 100.0,
        min_profit_usd: 1.0,
        base_currency_decimals: 8,
        gas_token_symbol: "ETH".to_string(),
        morpho: None,
        cross_token_map: HashMap::new(),
    }
}
