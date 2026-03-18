use crate::core::config::{AavePoolConfig, ChainConfig, MorphoDeployment, TokenInfo, MULTICALL3_ADDRESS};
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
    let rpc_url = std::env::var("HYPEREVM_RPC_URL")
        .unwrap_or_else(|_| "https://rpc.hyperliquid.xyz/evm".to_string());

    let liquidator = std::env::var("HYPEREVM_LIQUIDATOR_CONTRACT")
        .ok()
        .and_then(|s| s.parse::<Address>().ok())
        .or(Some(address!("17B7b1B73FFbA773E6A92Bcbc3b27538A427977c")));

    let tokens: HashMap<Address, TokenInfo> = HashMap::from([
        token(address!("5555555555555555555555555555555555555555"), "wHYPE", 18),
        token(address!("2416092f143378750bb29b79ed961ab195cceea5"), "ezETH", 18),
        token(address!("5d3a1ff2b6bab83b63cd9ad0787074081a52ef34"), "USDe", 18),
        token(address!("211cc4dd073734da055fbf44a2b4667d5e5fe5d2"), "sUSDe", 18),
        token(address!("9FDBdA0A5e284c32744D2f17Ee5c74B284993463"), "UBTC", 8),
        token(address!("Be6727B535545C67d5cAa73dEa54865B92CF7907"), "UETH", 18),
        token(address!("068f321Fa8Fb9f0D135f290Ef6a3e2813e1c8A29"), "USOL", 9),
        token(address!("b88339CB7199b77E23DB6E890353E22632Ba630f"), "USDC", 6),
        token(address!("B8CE59FC3717ada4C02eaDF9682A9e934F625ebb"), "USDT0", 6),
        token(address!("fd739d4e423301ce9385c1fb8850539d657c296d"), "kHYPE", 18),
        token(address!("94e8396e0869c9F2200760aF0621aFd240E1CF38"), "wstHYPE", 18),
        token(address!("d8FC8F0b03eBA61F64D08B0bef69d80916E5DdA9"), "beHYPE", 18),
        token(address!("b50A96253aBDF803D85efcDce07Ad8becBc52BD5"), "USDHL", 6),
        token(address!("111111a1a0667d36bd57c0a9f569b98057111111"), "USDH", 6),
        token(address!("0ad339d66bf4aed5ce31c64bc37b3244b6394a77"), "USR", 18),
    ]);

    let hyperlend = AavePoolConfig {
        label: "HyperLend".to_string(),
        pool: address!("00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b"),
        data_provider: address!("5481bf8d3946E6A3168640c1D7523eB59F055a29"),
        oracle: address!("C9Fb4fbE842d57EAc1dF3e641a281827493A630e"),
        liquidator_contract: liquidator,
        borrow_start_block: 20_000_000,
        is_v2: false,
        base_currency_decimals: 8,
        cross_token_map: HashMap::new(),
    };

    let hypurrfi = AavePoolConfig {
        label: "HypurrFi Pool".to_string(),
        pool: address!("cecce0eb9dd2ef7996e01e25dd70e461f918a14b"),
        data_provider: address!("895C799a5bbdCb63B80bEE5BD94E7b9138D977d6"),
        oracle: address!("9BE2ac1ff80950DCeb816842834930887249d9A8"),
        liquidator_contract: None,
        borrow_start_block: 10_000_000,
        is_v2: false,
        base_currency_decimals: 8,
        cross_token_map: HashMap::new(),
    };

    // HyperEVM WSS: Alchemy or fallback
    let wss_url = if rpc_url.contains("alchemy.com") {
        Some(rpc_url.replace("https://", "wss://"))
    } else {
        None // Public HyperEVM RPC doesn't have WSS
    };

    ChainConfig {
        chain_id: 999,
        name: "HyperEVM".to_string(),
        rpc_url,
        wss_url,
        enabled: true,
        aave_pools: vec![hyperlend.clone(), hypurrfi],
        // Legacy: primary pool = HyperLend
        aave_pool: hyperlend.pool,
        aave_data_provider: hyperlend.data_provider,
        aave_oracle: hyperlend.oracle,
        liquidator_contract: hyperlend.liquidator_contract,
        is_v2: false,
        multicall3: MULTICALL3_ADDRESS,
        borrow_start_block: 20_000_000,
        get_logs_chunk_size: 1_000, // HyperEVM getLogs limit = 1000 blocks
        scan_interval_ms: 30_000,
        poll_interval_ms: 3_000,
        tokens,
        min_debt_usd: 100.0,
        min_profit_usd: 1.0,
        base_currency_decimals: 8,
        gas_token_symbol: "HYPE".to_string(),
        morpho: Some(MorphoDeployment {
            morpho_address: address!("68e37dE8d93d3496ae143F2E900490f6280C57cD"),
            liquidator_contract: std::env::var("MORPHO_LIQUIDATOR_CONTRACT")
                .ok()
                .and_then(|s| s.parse::<Address>().ok()),
            start_block: 4_000_000, // Morpho deployment on HyperEVM
        }),
        cross_token_map: HashMap::new(),
    }
}
