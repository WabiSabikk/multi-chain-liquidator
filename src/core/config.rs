use alloy::primitives::Address;
use std::collections::HashMap;

/// Token metadata
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TokenInfo {
    pub address: Address,
    pub symbol: String,
    pub decimals: u8,
}

/// Aave V3 (or V2 fork) pool deployment on a chain
#[derive(Debug, Clone)]
pub struct AavePoolConfig {
    pub label: String,           // e.g. "Aave V3", "Lendle", "HypurrFi Pool"
    pub pool: Address,
    pub data_provider: Address,
    pub oracle: Address,
    pub liquidator_contract: Option<Address>,
    pub borrow_start_block: u64,
    /// True for Aave V2 forks (Lendle) — different Borrow event topic, no eMode
    pub is_v2: bool,
    /// Base currency decimals for USD: 8 for V3 (USD base), 18 for V2 (ETH base)
    pub base_currency_decimals: u8,
    /// For cross-pool liquidators: tokens that exist on this pool but NOT on the flash loan source.
    /// Maps debt token → (flash loan asset, flash loan asset decimals).
    /// When a candidate's debt token is in this map, executor uses cross-token liquidation.
    pub cross_token_map: HashMap<Address, CrossTokenConfig>,
}

/// Config for cross-token flash loan: when debt token isn't available on flash loan source
#[derive(Debug, Clone)]
pub struct CrossTokenConfig {
    /// Token to flash loan instead (must exist on Aave V3)
    pub flash_asset: Address,
    pub flash_asset_decimals: u8,
}

/// Chain-specific configuration for the liquidation bot
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub wss_url: Option<String>,
    pub enabled: bool,

    // Aave V3 pools (multiple per chain supported)
    pub aave_pools: Vec<AavePoolConfig>,

    // Legacy single-pool accessors (first pool = primary)
    pub aave_pool: Address,
    pub aave_data_provider: Address,
    pub aave_oracle: Address,

    // Liquidator contract (deployed by us)
    pub liquidator_contract: Option<Address>,

    /// True for Aave V2 forks (different event topic, no eMode)
    pub is_v2: bool,

    // Multicall3 (same on all EVM chains)
    pub multicall3: Address,

    // Scan parameters
    pub borrow_start_block: u64,
    pub get_logs_chunk_size: u64,
    /// Full scan interval (all borrowers)
    pub scan_interval_ms: u64,
    /// Block poll interval for quick scans (at-risk only)
    pub poll_interval_ms: u64,

    // Token registry
    pub tokens: HashMap<Address, TokenInfo>,

    // Minimum thresholds
    pub min_debt_usd: f64,
    pub min_profit_usd: f64,

    /// Base currency decimals for USD conversion (8 for Aave V3, 18 for V2/Lendle)
    pub base_currency_decimals: u8,
    /// Gas token symbol (MNT, ETH, HYPE)
    pub gas_token_symbol: String,

    // Morpho Blue (optional, HyperEVM only)
    pub morpho: Option<MorphoDeployment>,

    /// Cross-token flash loan map (for cross-pool liquidators like Lendle)
    pub cross_token_map: HashMap<Address, CrossTokenConfig>,
}

/// Load global settings from environment
pub struct GlobalConfig {
    pub private_key: String,
    pub dry_run: bool,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub dashboard_token: Option<String>,
    pub dashboard_port: u16,
}

impl GlobalConfig {
    pub fn from_env() -> eyre::Result<Self> {
        Ok(Self {
            private_key: std::env::var("PRIVATE_KEY")
                .map_err(|_| eyre::eyre!("Missing PRIVATE_KEY env var"))?,
            dry_run: std::env::var("DRY_RUN")
                .unwrap_or_else(|_| "true".into())
                .parse::<bool>()
                .unwrap_or(true),
            telegram_bot_token: std::env::var("TELEGRAM_BOT_TOKEN").ok(),
            telegram_chat_id: std::env::var("TELEGRAM_CHAT_ID").ok(),
            dashboard_token: std::env::var("DASHBOARD_TOKEN").ok(),
            dashboard_port: std::env::var("DASHBOARD_PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse::<u16>()
                .unwrap_or(3000),
        })
    }
}

/// Morpho Blue deployment on a chain
#[derive(Debug, Clone)]
pub struct MorphoDeployment {
    pub morpho_address: Address,
    pub liquidator_contract: Option<Address>,
    pub start_block: u64,
}

/// Common Multicall3 address (same on all EVM chains)
pub const MULTICALL3_ADDRESS: Address = alloy::primitives::address!("cA11bde05977b3631167028862bE2a173976CA11");
