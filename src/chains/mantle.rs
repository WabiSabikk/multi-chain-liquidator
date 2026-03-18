use crate::core::config::{AavePoolConfig, ChainConfig, CrossTokenConfig, TokenInfo, MULTICALL3_ADDRESS};
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
    let rpc_url = std::env::var("MANTLE_RPC_URL")
        .unwrap_or_else(|_| "https://mantle.drpc.org".to_string());

    let liquidator = std::env::var("MANTLE_LIQUIDATOR_CONTRACT")
        .ok()
        .and_then(|s| s.parse::<Address>().ok())
        .or(Some(address!("F2E6e7F255c46CC2353a8fD1D502f0c1920E1D43")));

    let tokens: HashMap<Address, TokenInfo> = HashMap::from([
        token(address!("dEAddEaDdeadDEadDEADDEAddEADDEAddead1111"), "WETH", 18),
        token(address!("78c1b0C915c4FAA5FffA6CAbf0219DA63d7f4cb8"), "WMNT", 18),
        token(address!("779Ded0c9e1022225f8E0630b35a9b54bE713736"), "USDT0", 6),
        token(address!("09Bc4E0D864854c6aFB6eB9A9cdF58aC190D0dF9"), "USDC", 6),
        token(address!("5d3a1Ff2b6BAb83b63cd9AD0787074081a52ef34"), "USDe", 18),
        token(address!("211Cc4DD073734dA055fbF44a2b4667d5E5fE5d2"), "sUSDe", 18),
        token(address!("C96dE26018A54D51c097160568752c4E3BD6C364"), "FBTC", 8),
        token(address!("cDA86A272531e8640cD7F1a92c01839911B90bb0"), "mETH", 18),
        token(address!("201EBa5CC46D216Ce6DC03F6a759e8E766e956aE"), "USDT", 6),
        token(address!("e6829d9a7ee3040e1276fa75293bde931859e8fa"), "cmETH", 18),
    ]);

    let aave_v3 = AavePoolConfig {
        label: "Aave V3".to_string(),
        pool: address!("458F293454fE0d67EC0655f3672301301DD51422"),
        data_provider: address!("487c5c669D9eee6057C44973207101276cf73b68"),
        oracle: address!("47a063CfDa980532267970d478EC340C0F80E8df"),
        liquidator_contract: liquidator,
        borrow_start_block: 91_300_000,
        is_v2: false,
        base_currency_decimals: 8,
        cross_token_map: HashMap::new(),
    };

    let lendle_liquidator = std::env::var("LENDLE_LIQUIDATOR_CONTRACT")
        .ok()
        .and_then(|s| s.parse::<Address>().ok())
        .or(Some(address!("f4C17331C8Dc453E8b5BAb98559FD7F1aA1cAD91"))); // v3: mode flag dispatch

    let lendle = AavePoolConfig {
        label: "Lendle".to_string(),
        pool: address!("CFa5aE7c2CE8Fadc6426C1ff872cA45378Fb7cF3"),
        data_provider: address!("552b9e4bae485C4B7F540777d7D25614CdB84773"),
        oracle: address!("870c9692Ab04944C86ec6FEeF63F261226506EfC"),
        liquidator_contract: lendle_liquidator,
        borrow_start_block: 91_500_000,
        is_v2: true,
        base_currency_decimals: 18, // V2 uses ETH base (18 decimals)
        cross_token_map: HashMap::from([
            // Lendle USDT not on Aave V3 → flash loan USDC instead
            (
                address!("201EBa5CC46D216Ce6DC03F6a759e8E766e956aE"), // Lendle USDT
                CrossTokenConfig {
                    flash_asset: address!("09Bc4E0D864854c6aFB6eB9A9cdF58aC190D0dF9"), // USDC
                    flash_asset_decimals: 6,
                },
            ),
        ]),
    };

    let wss_url = rpc_url.replace("https://", "wss://");

    ChainConfig {
        chain_id: 5000,
        name: "Mantle".to_string(),
        rpc_url,
        wss_url: Some(wss_url),
        enabled: true,
        aave_pools: vec![aave_v3.clone(), lendle],
        // Legacy: primary pool
        aave_pool: aave_v3.pool,
        aave_data_provider: aave_v3.data_provider,
        aave_oracle: aave_v3.oracle,
        liquidator_contract: aave_v3.liquidator_contract,
        is_v2: false,
        multicall3: MULTICALL3_ADDRESS,
        borrow_start_block: 91_300_000,
        get_logs_chunk_size: 10_000,
        scan_interval_ms: 30_000,
        poll_interval_ms: 3_000,
        tokens,
        min_debt_usd: 100.0,
        min_profit_usd: 1.0,
        base_currency_decimals: 8,
        gas_token_symbol: "MNT".to_string(),
        morpho: None,
        cross_token_map: HashMap::new(),
    }
}
