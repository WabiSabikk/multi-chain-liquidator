#![allow(dead_code)]

use alloy::primitives::{Address, U256};

/// Per-reserve configuration from Aave V3 on-chain data
#[derive(Debug, Clone)]
pub struct AssetConfig {
    pub address: Address,
    pub symbol: String,
    pub decimals: u8,
    /// 10^decimals
    pub unit: U256,
    /// In BPS (e.g. 8250 = 82.5%)
    pub liquidation_threshold: u64,
    /// In BPS (e.g. 10500 = 105%)
    pub liquidation_bonus: u64,
    /// In BPS
    pub ltv: u64,
    pub usage_as_collateral_enabled: bool,
    pub borrowing_enabled: bool,
    pub is_active: bool,
    pub is_frozen: bool,
    pub e_mode_category: u8,
}

/// E-mode category parameters
#[derive(Debug, Clone)]
pub struct EModeConfig {
    pub category_id: u8,
    pub label: String,
    pub liquidation_threshold: u64,
    pub liquidation_bonus: u64,
    pub ltv: u64,
    pub price_source: Address,
}

/// Per-asset data for a specific user
#[derive(Debug, Clone)]
pub struct UserReserveData {
    pub asset: Address,
    pub a_token_balance: U256,
    pub stable_debt: U256,
    pub variable_debt: U256,
    pub usage_as_collateral: bool,
}

/// Full position data for a borrower
#[derive(Debug, Clone)]
pub struct Position {
    pub address: Address,
    pub total_collateral_base: U256,
    pub total_debt_base: U256,
    pub avg_liquidation_threshold: U256,
    pub health_factor: U256,
    pub reserves: Vec<UserReserveData>,
    pub e_mode_category: u8,
    pub last_update_block: u64,
}

/// Candidate ready for liquidation execution
#[derive(Debug, Clone)]
pub struct LiquidationCandidate {
    pub address: Address,
    pub health_factor: f64,
    pub total_collateral_usd: f64,
    pub total_debt_usd: f64,
    pub collateral_asset: Address,
    pub collateral_symbol: String,
    pub collateral_decimals: u8,
    /// Oracle price of collateral in base currency units
    pub collateral_price: U256,
    pub debt_asset: Address,
    pub debt_symbol: String,
    pub debt_decimals: u8,
    /// Oracle price of debt in base currency units
    pub debt_price: U256,
    pub debt_to_cover: U256,
    pub estimated_profit_usd: f64,
    pub liquidation_bonus: u64,
    pub close_factor: U256,
    pub e_mode_category: u8,
}

/// Executor statistics
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ExecutorStats {
    pub total_candidates: u64,
    pub total_attempted: u64,
    pub total_success: u64,
    pub total_failed: u64,
    pub total_skipped: u64,
    pub total_sim_failed: u64,
    pub total_profit_usd: f64,
}

#[allow(dead_code)]
pub const BASE_CURRENCY_DECIMALS: u8 = 8;
#[allow(dead_code)]
pub const BASE_CURRENCY_UNIT: u64 = 100_000_000;
pub const HF_ONE: u128 = 1_000_000_000_000_000_000;
#[allow(dead_code)]
pub const HF_CLOSE_FACTOR_THRESHOLD: u128 = 950_000_000_000_000_000;
pub const BPS_PRECISION: u64 = 10_000;
