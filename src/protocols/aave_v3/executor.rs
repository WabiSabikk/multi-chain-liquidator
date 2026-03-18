use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::core::alerts::send_telegram_alert;
use crate::core::config::CrossTokenConfig;
use crate::core::dashboard::{now_unix, DashboardEvent, DashboardState};
use crate::core::types::{ExecutorStats, LiquidationCandidate};
use crate::dex::odos;
use crate::protocols::aave_v3::contracts::IFlashLoanLiquidator;

/// Aave V3 error codes
fn aave_error_name(code: &str) -> Option<&'static str> {
    match code {
        "35" => Some("HEALTH_FACTOR_NOT_BELOW_THRESHOLD"),
        "36" => Some("COLLATERAL_CANNOT_BE_LIQUIDATED"),
        "38" => Some("SPECIFIED_CURRENCY_NOT_BORROWED_BY_USER"),
        "39" => Some("NOT_ENOUGH_LIQUIDITY_TO_LIQUIDATE"),
        "40" => Some("NO_DEBT_OF_SELECTED_TYPE"),
        "41" => Some("NO_EXPLICIT_AMOUNT_TO_LIQUIDATE"),
        "42" => Some("LIQUIDATION_CLOSE_FACTOR_DEPRECATED"),
        _ => None,
    }
}

pub struct AaveV3Executor {
    chain_name: String,
    chain_id: u64,
    liquidator_contract: Option<Address>,
    dry_run: bool,
    min_profit_usd: f64,

    stats: ExecutorStats,
    cooldowns: HashMap<Address, Instant>,
    cooldown_success: std::time::Duration,
    #[allow(dead_code)]
    cooldown_sim_fail: std::time::Duration,

    // Nonce tracking
    current_nonce: Option<u64>,

    // Telegram (legacy, kept for backward compat)
    telegram_bot_token: Option<String>,
    telegram_chat_id: Option<String>,

    // Dashboard
    dashboard: Arc<RwLock<DashboardState>>,

    // Cross-token flash loan map (for Lendle USDT etc.)
    cross_token_map: HashMap<Address, CrossTokenConfig>,
}

impl AaveV3Executor {
    pub fn new(
        chain_name: String,
        chain_id: u64,
        liquidator_contract: Option<Address>,
        dry_run: bool,
        min_profit_usd: f64,
        telegram_bot_token: Option<String>,
        telegram_chat_id: Option<String>,
        dashboard: Arc<RwLock<DashboardState>>,
        cross_token_map: HashMap<Address, CrossTokenConfig>,
    ) -> Self {
        Self {
            chain_name,
            chain_id,
            liquidator_contract,
            dry_run,
            min_profit_usd,
            stats: ExecutorStats::default(),
            cooldowns: HashMap::new(),
            cooldown_success: std::time::Duration::from_secs(120),
            cooldown_sim_fail: std::time::Duration::from_secs(30),
            current_nonce: None,
            telegram_bot_token,
            telegram_chat_id,
            dashboard,
            cross_token_map,
        }
    }

    pub async fn process_candidates<P: Provider + Clone>(
        &mut self,
        provider: &P,
        signer_address: Address,
        candidates: &[LiquidationCandidate],
    ) {
        if candidates.is_empty() {
            return;
        }
        self.stats.total_candidates += candidates.len() as u64;

        // Filter: cooldown + min profit
        let mut filtered: Vec<LiquidationCandidate> = Vec::new();
        for candidate in candidates {
            if let Some(last) = self.cooldowns.get(&candidate.address) {
                if last.elapsed() < self.cooldown_success {
                    self.stats.total_skipped += 1;
                    continue;
                }
            }
            if candidate.estimated_profit_usd < self.min_profit_usd {
                self.stats.total_skipped += 1;
                continue;
            }
            filtered.push(candidate.clone());
        }

        if filtered.is_empty() {
            return;
        }

        if self.dry_run {
            for c in &filtered {
                self.dry_run_execute(provider, signer_address, c).await;
            }
            return;
        }

        // Single candidate — sequential path (most common case)
        if filtered.len() == 1 {
            self.live_execute(provider, signer_address, &filtered[0]).await;
            return;
        }

        // Phase 6.3: Multiple candidates — parallel simulation + sequential execution
        self.parallel_execute(provider, signer_address, &filtered).await;
    }

    /// Phase 6.3: Parallel simulation for multiple simultaneous candidates.
    /// Simulates all candidates in parallel, then executes successful ones
    /// sequentially with pre-allocated nonces.
    async fn parallel_execute<P: Provider + Clone>(
        &mut self,
        provider: &P,
        signer_address: Address,
        candidates: &[LiquidationCandidate],
    ) {
        let Some(contract_addr) = self.liquidator_contract else {
            error!(chain = %self.chain_name, "No LIQUIDATOR_CONTRACT configured");
            return;
        };

        info!(
            chain = %self.chain_name,
            count = candidates.len(),
            "Parallel liquidation: simulating {} candidates",
            candidates.len()
        );

        let sim_start = Instant::now();

        // Simulate each candidate (with Odos routing if available)
        let mut executable: Vec<(&LiquidationCandidate, Option<odos::OdosRoute>)> = Vec::new();
        for c in candidates {
            let sim = self
                .simulate_liquidation(provider, contract_addr, signer_address, c)
                .await;
            if sim.success {
                executable.push((c, sim.route));
            } else {
                self.stats.total_sim_failed += 1;
                self.cooldowns.insert(c.address, Instant::now());
                warn!(
                    chain = %self.chain_name,
                    target = %c.address,
                    reason = %sim.reason,
                    "Parallel sim FAILED"
                );
            }
        }
        let sim_ms = sim_start.elapsed().as_millis();

        info!(
            chain = %self.chain_name,
            sim_ms,
            passed = executable.len(),
            failed = candidates.len() - executable.len(),
            "Parallel simulation complete"
        );

        if executable.is_empty() {
            return;
        }

        // Get base nonce
        let base_nonce = match self.current_nonce {
            Some(n) => n,
            None => match provider.get_transaction_count(signer_address).await {
                Ok(n) => n,
                Err(e) => {
                    error!(chain = %self.chain_name, "Nonce FAILED: {e}");
                    self.stats.total_failed += executable.len() as u64;
                    return;
                }
            },
        };

        // Execute with pre-allocated nonces (sequential for nonce safety)
        for (i, (candidate, route)) in executable.iter().enumerate() {
            let nonce = base_nonce + i as u64;
            let t0 = Instant::now();
            self.stats.total_attempted += 1;
            self.cooldowns.insert(candidate.address, Instant::now());

            let routing = if route.is_some() { "Odos" } else { "default" };
            info!(
                chain = %self.chain_name,
                target = %candidate.address,
                hf = format!("{:.4}", candidate.health_factor),
                pair = format!("{} -> {}", candidate.debt_symbol, candidate.collateral_symbol),
                debt_usd = format!("${:.0}", candidate.total_debt_usd),
                routing,
                nonce,
                "[{}/{}] LIQUIDATION ATTEMPT",
                i + 1,
                executable.len()
            );

            let contract = IFlashLoanLiquidator::new(contract_addr, provider.clone());

            let (gas_limit, send_result) = if let Some(r) = route {
                let call = contract.executeLiquidation_1(
                    candidate.collateral_asset, candidate.debt_asset, candidate.address,
                    candidate.debt_to_cover, U256::ZERO, r.calldata.clone(), r.router,
                );
                let gas = match call.estimate_gas().await {
                    Ok(est) => est + est / 5,
                    Err(_) => 2_000_000,
                };
                (gas, call.gas(gas).nonce(nonce).send().await)
            } else {
                let call = contract.executeLiquidation_0(
                    candidate.collateral_asset, candidate.debt_asset, candidate.address,
                    candidate.debt_to_cover, U256::ZERO,
                );
                let gas = match call.estimate_gas().await {
                    Ok(est) => est + est / 5,
                    Err(_) => 2_000_000,
                };
                (gas, call.gas(gas).nonce(nonce).send().await)
            };
            let _ = gas_limit;

            match send_result {
                Ok(pending) => {
                    let send_ms = t0.elapsed().as_millis();
                    let tx_hash = format!("{}", pending.tx_hash());
                    info!(chain = %self.chain_name, ms = send_ms, tx = %tx_hash, "TX sent");

                    match pending.get_receipt().await {
                        Ok(receipt) => {
                            let total_ms = t0.elapsed().as_millis();
                            if receipt.status() {
                                self.stats.total_success += 1;
                                self.stats.total_profit_usd += candidate.estimated_profit_usd;
                                info!(
                                    chain = %self.chain_name,
                                    tx = %tx_hash,
                                    block = receipt.block_number.unwrap_or(0),
                                    gas_used = receipt.gas_used,
                                    total_ms,
                                    "SUCCESS"
                                );
                                self.alert(candidate, "SUCCESS", Some(&tx_hash)).await;
                            } else {
                                self.stats.total_failed += 1;
                                error!(
                                    chain = %self.chain_name,
                                    tx = %tx_hash,
                                    total_ms,
                                    "REVERTED"
                                );
                                self.alert(candidate, "REVERTED", Some(&tx_hash)).await;
                            }
                        }
                        Err(e) => {
                            self.stats.total_failed += 1;
                            error!(chain = %self.chain_name, tx = %tx_hash, "Receipt FAILED: {e}");
                        }
                    }
                }
                Err(e) => {
                    self.stats.total_failed += 1;
                    let reason = parse_revert_reason(&e.to_string());
                    error!(chain = %self.chain_name, reason = %reason, "TX FAILED");
                    if e.to_string().contains("nonce") || e.to_string().contains("NONCE") {
                        self.current_nonce = None;
                        info!(chain = %self.chain_name, "Nonce reset — aborting parallel batch");
                        break;
                    }
                }
            }
        }

        // Update nonce counter
        self.current_nonce = Some(base_nonce + executable.len() as u64);
    }

    async fn dry_run_execute<P: Provider + Clone>(
        &mut self,
        provider: &P,
        signer_address: Address,
        candidate: &LiquidationCandidate,
    ) {
        let t0 = Instant::now();

        info!(
            chain = %self.chain_name,
            target = %candidate.address,
            hf = format!("{:.4}", candidate.health_factor),
            e_mode = candidate.e_mode_category,
            pair = format!("{} -> {}", candidate.debt_symbol, candidate.collateral_symbol),
            debt_usd = format!("${:.0}", candidate.total_debt_usd),
            profit_usd = format!("${:.2}", candidate.estimated_profit_usd),
            "[DRY RUN] LIQUIDATION CANDIDATE"
        );

        // Simulate even in dry run
        if let Some(contract_addr) = self.liquidator_contract {
            let sim_result = self
                .simulate_liquidation(provider, contract_addr, signer_address, candidate)
                .await;
            let sim_ms = t0.elapsed().as_millis();

            if sim_result.success {
                info!(chain = %self.chain_name, ms = sim_ms, "Simulation OK");
            } else {
                info!(
                    chain = %self.chain_name,
                    ms = sim_ms,
                    reason = %sim_result.reason,
                    "Simulation FAILED"
                );
            }
        }

        if candidate.estimated_profit_usd > 10.0 {
            self.alert(candidate, "DRY_RUN", None).await;
        }

        self.cooldowns.insert(candidate.address, Instant::now());
    }

    async fn live_execute<P: Provider + Clone>(
        &mut self,
        provider: &P,
        signer_address: Address,
        candidate: &LiquidationCandidate,
    ) {
        let Some(contract_addr) = self.liquidator_contract else {
            error!(
                chain = %self.chain_name,
                "No LIQUIDATOR_CONTRACT configured — cannot execute LIVE"
            );
            return;
        };

        let t0 = Instant::now();

        info!(
            chain = %self.chain_name,
            target = %candidate.address,
            hf = format!("{:.4}", candidate.health_factor),
            pair = format!("{} -> {}", candidate.debt_symbol, candidate.collateral_symbol),
            debt_usd = format!("${:.0}", candidate.total_debt_usd),
            profit_usd = format!("${:.2}", candidate.estimated_profit_usd),
            "LIQUIDATION ATTEMPT"
        );

        // Phase 1: Simulation
        let sim = self
            .simulate_liquidation(provider, contract_addr, signer_address, candidate)
            .await;
        let sim_ms = t0.elapsed().as_millis();

        if !sim.success {
            self.stats.total_sim_failed += 1;
            self.cooldowns.insert(candidate.address, Instant::now());
            warn!(
                chain = %self.chain_name,
                ms = sim_ms,
                reason = %sim.reason,
                "Phase 1: Simulation FAILED"
            );
            return;
        }
        let is_cross_token = self.cross_token_map.contains_key(&candidate.debt_asset);
        let odos_route = sim.route;
        info!(
            chain = %self.chain_name,
            ms = sim_ms,
            routing = if is_cross_token { "cross-token" } else if odos_route.is_some() { "Odos" } else { "default" },
            "Phase 1: Simulation OK"
        );

        // Phase 2: Get nonce
        let nonce = match self.current_nonce {
            Some(n) => n,
            None => {
                match provider.get_transaction_count(signer_address).await {
                    Ok(n) => {
                        self.current_nonce = Some(n);
                        n
                    }
                    Err(e) => {
                        error!(chain = %self.chain_name, "Phase 2: Nonce FAILED: {e}");
                        self.stats.total_failed += 1;
                        return;
                    }
                }
            }
        };
        info!(chain = %self.chain_name, nonce, "Phase 2: Nonce");

        // Phase 3: Build & Send TX
        self.stats.total_attempted += 1;
        self.cooldowns.insert(candidate.address, Instant::now());

        let contract = IFlashLoanLiquidator::new(contract_addr, provider.clone());

        // Phase 3: Build & Send TX
        // Three paths: cross-token, Odos routing, or default DEX
        let send_result = if is_cross_token {
            // Re-simulate cross-token to get fresh Odos routes (they expire quickly)
            let ct = self.cross_token_map.get(&candidate.debt_asset).unwrap().clone();
            let sim2 = self.simulate_cross_token(provider, contract_addr, signer_address, candidate, &ct).await;
            if !sim2.success {
                error!(chain = %self.chain_name, reason = %sim2.reason, "Cross-token re-sim failed");
                self.stats.total_failed += 1;
                return;
            }
            // Build the cross-token call with fresh routes
            // Note: we need the routes from simulate_cross_token, but it doesn't return them.
            // For now, re-fetch routes inline (this duplicates work but is correct).
            let debt_unit = U256::from(10u64).pow(U256::from(candidate.debt_decimals as u64));
            let flash_unit = U256::from(10u64).pow(U256::from(ct.flash_asset_decimals as u64));
            let flash_amount = candidate.debt_to_cover * U256::from(102u64) * flash_unit / (U256::from(100u64) * debt_unit);

            let pre = odos::get_route(self.chain_id, ct.flash_asset, candidate.debt_asset, flash_amount, contract_addr).await;
            let col_unit = U256::from(10u64).pow(U256::from(candidate.collateral_decimals as u64));
            let value_in_base = candidate.debt_to_cover * candidate.debt_price / debt_unit;
            let bonus = if candidate.liquidation_bonus > 0 { candidate.liquidation_bonus } else { 10500u64 };
            let est_col = value_in_base * U256::from(bonus) * col_unit / (candidate.collateral_price * U256::from(10000u64)) * U256::from(90u64) / U256::from(100u64);
            let post = odos::get_route(self.chain_id, candidate.collateral_asset, ct.flash_asset, est_col, contract_addr).await;

            if let (Some(pre_r), Some(post_r)) = (pre, post) {
                let call = contract.executeCrossTokenLiquidation(
                    candidate.collateral_asset, candidate.debt_asset, candidate.address,
                    candidate.debt_to_cover, ct.flash_asset, flash_amount,
                    pre_r.calldata, pre_r.router, post_r.calldata, post_r.router,
                );
                let gas = match call.estimate_gas().await {
                    Ok(est) => est + est / 5,
                    Err(_) => 3_000_000,
                };
                call.gas(gas).nonce(nonce).send().await
            } else {
                error!(chain = %self.chain_name, "Cross-token: Odos routes unavailable at TX time");
                self.stats.total_failed += 1;
                return;
            }
        } else if let Some(ref route) = odos_route {
            let call = contract.executeLiquidation_1(
                candidate.collateral_asset, candidate.debt_asset, candidate.address,
                candidate.debt_to_cover, U256::ZERO, route.calldata.clone(), route.router,
            );
            let gas = match call.estimate_gas().await {
                Ok(est) => est + est / 5,
                Err(_) => 2_000_000,
            };
            call.gas(gas).nonce(nonce).send().await
        } else {
            let call = contract.executeLiquidation_0(
                candidate.collateral_asset, candidate.debt_asset, candidate.address,
                candidate.debt_to_cover, U256::ZERO,
            );
            let gas = match call.estimate_gas().await {
                Ok(est) => est + est / 5,
                Err(_) => 2_000_000,
            };
            call.gas(gas).nonce(nonce).send().await
        };

        match send_result {
            Ok(pending) => {
                let send_ms = t0.elapsed().as_millis();
                let tx_hash = format!("{}", pending.tx_hash());
                info!(
                    chain = %self.chain_name,
                    ms = send_ms,
                    tx = %tx_hash,
                    "Phase 3: TX sent"
                );
                self.current_nonce = Some(nonce + 1);

                // Phase 4: Wait for confirmation
                match pending.get_receipt().await {
                    Ok(receipt) => {
                        let total_ms = t0.elapsed().as_millis();
                        if receipt.status() {
                            self.stats.total_success += 1;
                            self.stats.total_profit_usd += candidate.estimated_profit_usd;
                            info!(
                                chain = %self.chain_name,
                                tx = %tx_hash,
                                block = receipt.block_number.unwrap_or(0),
                                gas_used = receipt.gas_used,
                                total_ms,
                                "SUCCESS"
                            );
                            self.alert(candidate, "SUCCESS", Some(&tx_hash)).await;
                        } else {
                            self.stats.total_failed += 1;
                            error!(
                                chain = %self.chain_name,
                                tx = %tx_hash,
                                total_ms,
                                "REVERTED"
                            );
                            self.alert(candidate, "REVERTED", Some(&tx_hash)).await;
                        }
                    }
                    Err(e) => {
                        self.stats.total_failed += 1;
                        error!(
                            chain = %self.chain_name,
                            tx = %tx_hash,
                            "Receipt FAILED: {e}"
                        );
                    }
                }
            }
            Err(e) => {
                self.stats.total_failed += 1;
                let reason = parse_revert_reason(&e.to_string());
                error!(
                    chain = %self.chain_name,
                    reason = %reason,
                    "TX FAILED"
                );

                // Reset nonce on nonce errors
                let err_msg = e.to_string();
                if err_msg.contains("nonce") || err_msg.contains("NONCE") {
                    self.current_nonce = None;
                    info!(chain = %self.chain_name, "Nonce reset due to nonce error");
                }
            }
        }
    }

    /// Try to get aggregator routing for a collateral→debt swap.
    /// Estimates collateral amount from debt_to_cover, liquidation bonus, and decimal adjustment.
    async fn try_aggregator_route(
        &self,
        candidate: &LiquidationCandidate,
        contract_addr: Address,
    ) -> Option<odos::OdosRoute> {
        if candidate.collateral_asset == candidate.debt_asset {
            return None; // same token, no swap needed
        }
        if !odos::is_supported(self.chain_id) {
            return None;
        }

        // Estimate collateral received from liquidation using oracle prices:
        // col_amount = debt_to_cover * debt_price / col_price * (bonus / 10000)
        // Adjust for decimals: * col_unit / debt_unit
        let col_price = candidate.collateral_price;
        let debt_price = candidate.debt_price;

        if col_price.is_zero() || debt_price.is_zero() {
            warn!(chain = %self.chain_name, "Odos: missing oracle prices, skipping");
            return None;
        }

        let bonus_bps = if candidate.liquidation_bonus > 0 {
            candidate.liquidation_bonus
        } else {
            10500u64
        };

        let col_unit = U256::from(10u64).pow(U256::from(candidate.collateral_decimals as u64));
        let debt_unit = U256::from(10u64).pow(U256::from(candidate.debt_decimals as u64));

        // value_in_base = debt_to_cover * debt_price / debt_unit
        // col_amount = value_in_base * bonus * col_unit / (col_price * 10000)
        let value_in_base = candidate.debt_to_cover * debt_price / debt_unit;
        let estimated_collateral =
            value_in_base * U256::from(bonus_bps) * col_unit / (col_price * U256::from(10000u64));

        // 90% safety margin — ensure amountIn <= actual balance after liquidation
        let conservative = estimated_collateral * U256::from(90u64) / U256::from(100u64);

        if conservative.is_zero() {
            return None;
        }

        info!(
            chain = %self.chain_name,
            col = %candidate.collateral_symbol,
            debt = %candidate.debt_symbol,
            col_dec = candidate.collateral_decimals,
            debt_dec = candidate.debt_decimals,
            estimated = %conservative,
            "Odos: estimating collateral for swap"
        );

        odos::get_route(
            self.chain_id,
            candidate.collateral_asset,
            candidate.debt_asset,
            conservative,
            contract_addr,
        )
        .await
    }

    async fn simulate_liquidation<P: Provider + Clone>(
        &self,
        provider: &P,
        contract_addr: Address,
        signer_address: Address,
        candidate: &LiquidationCandidate,
    ) -> SimResult {
        let contract = IFlashLoanLiquidator::new(contract_addr, provider.clone());

        // Check if this is a cross-token case (e.g. Lendle USDT debt)
        if let Some(ct) = self.cross_token_map.get(&candidate.debt_asset) {
            return self
                .simulate_cross_token(provider, contract_addr, signer_address, candidate, ct)
                .await;
        }

        // Try aggregator routing first
        if let Some(route) = self.try_aggregator_route(candidate, contract_addr).await {
            info!(
                chain = %self.chain_name,
                router = %route.router,
                "Simulating with Odos routing"
            );
            let call = contract.executeLiquidation_1(
                candidate.collateral_asset,
                candidate.debt_asset,
                candidate.address,
                candidate.debt_to_cover,
                U256::ZERO,
                route.calldata.clone(),
                route.router,
            );
            match call.from(signer_address).call().await {
                Ok(_) => {
                    return SimResult {
                        success: true,
                        reason: "OK (Odos)".to_string(),
                        route: Some(route),
                    };
                }
                Err(e) => {
                    let reason = parse_revert_reason(&e.to_string());
                    warn!(
                        chain = %self.chain_name,
                        reason = %reason,
                        "Odos route sim failed, trying default"
                    );
                }
            }
        }

        // Fallback: default DEX path (5-param)
        let call = contract.executeLiquidation_0(
            candidate.collateral_asset,
            candidate.debt_asset,
            candidate.address,
            candidate.debt_to_cover,
            U256::ZERO,
        );

        match call.from(signer_address).call().await {
            Ok(_) => SimResult {
                success: true,
                reason: "OK".to_string(),
                route: None,
            },
            Err(e) => SimResult {
                success: false,
                reason: parse_revert_reason(&e.to_string()),
                route: None,
            },
        }
    }

    /// Simulate cross-token liquidation: flash loan different asset, swap to debt, liquidate
    async fn simulate_cross_token<P: Provider + Clone>(
        &self,
        _provider: &P,
        contract_addr: Address,
        _signer_address: Address,
        candidate: &LiquidationCandidate,
        ct: &CrossTokenConfig,
    ) -> SimResult {
        // Need two Odos routes:
        // 1. pre-swap: flashAsset → debtAsset (before liquidation)
        // 2. post-swap: collateral → flashAsset (after liquidation)

        if !odos::is_supported(self.chain_id) {
            return SimResult {
                success: false,
                reason: "Cross-token needs Odos but chain not supported".to_string(),
                route: None,
            };
        }

        // Estimate flash loan amount: debt_to_cover converted to flash asset units
        // flash_amount = debt_to_cover * debt_price / flash_price * flash_unit / debt_unit
        let debt_price = candidate.debt_price;
        let flash_unit = U256::from(10u64).pow(U256::from(ct.flash_asset_decimals as u64));
        let debt_unit = U256::from(10u64).pow(U256::from(candidate.debt_decimals as u64));

        // For USDC(6)→USDT(6): amount ≈ debt_to_cover * 1.02 (2% buffer for swap slippage)
        let flash_amount = candidate.debt_to_cover * U256::from(102u64) * flash_unit
            / (U256::from(100u64) * debt_unit);

        // Get pre-swap route: flashAsset → debtAsset
        let pre_route = odos::get_route(
            self.chain_id,
            ct.flash_asset,
            candidate.debt_asset,
            flash_amount,
            contract_addr,
        )
        .await;

        let Some(pre) = pre_route else {
            return SimResult {
                success: false,
                reason: "Odos: no pre-swap route for cross-token".to_string(),
                route: None,
            };
        };

        // Estimate collateral for post-swap
        let col_price = candidate.collateral_price;
        if col_price.is_zero() || debt_price.is_zero() {
            return SimResult {
                success: false,
                reason: "Missing oracle prices for cross-token".to_string(),
                route: None,
            };
        }

        let bonus_bps = if candidate.liquidation_bonus > 0 {
            candidate.liquidation_bonus
        } else {
            10500u64
        };
        let col_unit = U256::from(10u64).pow(U256::from(candidate.collateral_decimals as u64));
        let value_in_base = candidate.debt_to_cover * debt_price / debt_unit;
        let est_col = value_in_base * U256::from(bonus_bps) * col_unit
            / (col_price * U256::from(10000u64));
        let est_col_90 = est_col * U256::from(90u64) / U256::from(100u64);

        // Get post-swap route: collateral → flashAsset
        let post_route = odos::get_route(
            self.chain_id,
            candidate.collateral_asset,
            ct.flash_asset,
            est_col_90,
            contract_addr,
        )
        .await;

        let Some(post) = post_route else {
            return SimResult {
                success: false,
                reason: "Odos: no post-swap route for cross-token".to_string(),
                route: None,
            };
        };

        info!(
            chain = %self.chain_name,
            flash_asset = %ct.flash_asset,
            flash_amount = %flash_amount,
            "Simulating cross-token liquidation"
        );

        // Simulate the full cross-token call
        let contract = IFlashLoanLiquidator::new(contract_addr, _provider.clone());
        let call = contract.executeCrossTokenLiquidation(
            candidate.collateral_asset,
            candidate.debt_asset,
            candidate.address,
            candidate.debt_to_cover,
            ct.flash_asset,
            flash_amount,
            pre.calldata.clone(),
            pre.router,
            post.calldata.clone(),
            post.router,
        );

        match call.from(_signer_address).call().await {
            Ok(_) => SimResult {
                success: true,
                reason: "OK (cross-token)".to_string(),
                route: None, // cross-token uses its own path, not standard Odos route
            },
            Err(e) => SimResult {
                success: false,
                reason: format!("cross-token sim: {}", parse_revert_reason(&e.to_string())),
                route: None,
            },
        }
    }

    async fn alert(
        &self,
        candidate: &LiquidationCandidate,
        status: &str,
        tx_hash: Option<&str>,
    ) {
        // Push to dashboard
        let event_type = match status {
            "SUCCESS" => "liquidation_success",
            "REVERTED" => "liquidation_failed",
            "DRY_RUN" => "candidate_found",
            _ => "info",
        };
        let message = format!(
            "HF={:.4} {} -> {} debt=${:.0} profit=${:.2}{}",
            candidate.health_factor,
            candidate.debt_symbol,
            candidate.collateral_symbol,
            candidate.total_debt_usd,
            candidate.estimated_profit_usd,
            tx_hash.map(|h| format!(" TX={}", &h[..10.min(h.len())])).unwrap_or_default(),
        );
        {
            let mut state = self.dashboard.write().await;
            state.push_event(DashboardEvent {
                timestamp_unix: now_unix(),
                chain: self.chain_name.clone(),
                event_type: event_type.to_string(),
                message,
                data: None,
            });
        }

        // Also send Telegram if configured (backward compat)
        if let (Some(token), Some(chat_id)) = (&self.telegram_bot_token, &self.telegram_chat_id) {
            send_telegram_alert(token, chat_id, &self.chain_name, candidate, status, tx_hash)
                .await;
        }
    }

    pub fn clean_cooldowns(&mut self) {
        self.cooldowns
            .retain(|_, time| time.elapsed() < self.cooldown_success);
    }

    pub fn stats(&self) -> &ExecutorStats {
        &self.stats
    }
}

struct SimResult {
    success: bool,
    reason: String,
    route: Option<odos::OdosRoute>,
}

fn parse_revert_reason(err_msg: &str) -> String {
    // Aave V3 errors are require() with short numeric strings like "35".
    // Geth nodes decode them as "execution reverted: 35" in the RPC message.
    // Match "reverted: XX" to avoid false positives from hex data, gas amounts, etc.
    if let Some(pos) = err_msg.find("reverted: ") {
        let after = &err_msg[pos + 10..];
        let end = after.find(|c: char| !c.is_ascii_digit()).unwrap_or(after.len());
        let code = &after[..end];
        if !code.is_empty() {
            if let Some(name) = aave_error_name(code) {
                return format!("AAVE: {name} (code {code})");
            }
        }
    }

    if err_msg.contains("HEALTH_FACTOR") {
        return "HEALTH_FACTOR_NOT_BELOW_THRESHOLD".to_string();
    }
    if err_msg.contains("InsufficientProfit") {
        return "INSUFFICIENT_PROFIT (swap slippage too high)".to_string();
    }
    if err_msg.contains("SwapFailed") {
        return "SWAP_FAILED (DEX liquidity issue)".to_string();
    }
    if err_msg.contains("insufficient funds") {
        return "INSUFFICIENT_GAS".to_string();
    }
    if err_msg.contains("execution reverted") {
        return "EXECUTION_REVERTED".to_string();
    }
    if err_msg.contains("timeout") {
        return "RPC_TIMEOUT".to_string();
    }

    err_msg.chars().take(100).collect()
}
