use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::core::dashboard::{now_unix, DashboardEvent, DashboardState};
use crate::core::types::ExecutorStats;
use crate::protocols::morpho::contracts::IMorphoLiquidator;
use crate::protocols::morpho::monitor::MorphoCandidate;

pub struct MorphoExecutor {
    chain_name: String,
    liquidator_contract: Option<Address>,
    dry_run: bool,
    min_profit_usd: f64,
    stats: ExecutorStats,
    cooldowns: HashMap<(alloy::primitives::FixedBytes<32>, Address), Instant>,
    cooldown_duration: std::time::Duration,
    current_nonce: Option<u64>,
    dashboard: Arc<RwLock<DashboardState>>,
}

impl MorphoExecutor {
    pub fn new(
        chain_name: String,
        liquidator_contract: Option<Address>,
        dry_run: bool,
        min_profit_usd: f64,
        dashboard: Arc<RwLock<DashboardState>>,
    ) -> Self {
        Self {
            chain_name,
            liquidator_contract,
            dry_run,
            min_profit_usd,
            stats: ExecutorStats::default(),
            cooldowns: HashMap::new(),
            cooldown_duration: std::time::Duration::from_secs(120),
            current_nonce: None,
            dashboard,
        }
    }

    pub async fn process_candidates<P: Provider + Clone>(
        &mut self,
        provider: &P,
        signer_address: Address,
        candidates: &[MorphoCandidate],
    ) {
        if candidates.is_empty() {
            return;
        }
        self.stats.total_candidates += candidates.len() as u64;

        // Filter: cooldown + min profit
        let mut filtered: Vec<MorphoCandidate> = Vec::new();
        for candidate in candidates {
            let key = (candidate.market_id, candidate.borrower);
            if let Some(last) = self.cooldowns.get(&key) {
                if last.elapsed() < self.cooldown_duration {
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
                self.dry_run_execute(c).await;
            }
            return;
        }

        // Single candidate — sequential path
        if filtered.len() == 1 {
            self.live_execute(provider, signer_address, &filtered[0]).await;
            self.cooldowns.insert((filtered[0].market_id, filtered[0].borrower), Instant::now());
            return;
        }

        // Phase 6.3: Multiple candidates — parallel simulation
        self.parallel_execute(provider, signer_address, &filtered).await;
    }

    /// Phase 6.3: Parallel simulation for multiple Morpho candidates.
    async fn parallel_execute<P: Provider + Clone>(
        &mut self,
        provider: &P,
        signer_address: Address,
        candidates: &[MorphoCandidate],
    ) {
        let Some(contract_addr) = self.liquidator_contract else {
            error!(chain = %self.chain_name, "No Morpho liquidator contract configured");
            return;
        };

        info!(
            chain = %self.chain_name,
            count = candidates.len(),
            "Morpho parallel: simulating {} candidates",
            candidates.len()
        );

        let sim_start = Instant::now();

        // Simulate all in parallel
        let sim_futures: Vec<_> = candidates
            .iter()
            .map(|c| {
                let provider = provider.clone();
                let c = c.clone();
                async move {
                    let contract = IMorphoLiquidator::new(contract_addr, provider);
                    let call = contract.liquidate(
                        c.loan_token,
                        c.collateral_token,
                        c.oracle,
                        c.irm,
                        c.lltv,
                        c.borrower,
                        c.seized_assets,
                        U256::ZERO,
                        alloy::primitives::Bytes::new(),
                        Address::ZERO,
                    );
                    match call.from(signer_address).call().await {
                        Ok(_) => (true, "OK".to_string()),
                        Err(e) => (false, e.to_string()),
                    }
                }
            })
            .collect();

        let sim_results = futures_util::future::join_all(sim_futures).await;
        let sim_ms = sim_start.elapsed().as_millis();

        let mut executable: Vec<&MorphoCandidate> = Vec::new();
        for (i, (success, reason)) in sim_results.iter().enumerate() {
            if *success {
                executable.push(&candidates[i]);
            } else {
                self.stats.total_sim_failed += 1;
                self.cooldowns.insert((candidates[i].market_id, candidates[i].borrower), Instant::now());
                warn!(chain = %self.chain_name, target = %candidates[i].borrower, reason = %reason, "Morpho parallel sim FAILED");
            }
        }

        info!(
            chain = %self.chain_name,
            sim_ms,
            passed = executable.len(),
            failed = candidates.len() - executable.len(),
            "Morpho parallel simulation complete"
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
                    error!(chain = %self.chain_name, "Nonce failed: {e}");
                    self.stats.total_failed += executable.len() as u64;
                    return;
                }
            },
        };

        for (i, candidate) in executable.iter().enumerate() {
            let nonce = base_nonce + i as u64;
            self.stats.total_attempted += 1;
            self.cooldowns.insert((candidate.market_id, candidate.borrower), Instant::now());

            let contract = IMorphoLiquidator::new(contract_addr, provider.clone());
            let call = contract.liquidate(
                candidate.loan_token,
                candidate.collateral_token,
                candidate.oracle,
                candidate.irm,
                candidate.lltv,
                candidate.borrower,
                candidate.seized_assets,
                U256::ZERO,
                alloy::primitives::Bytes::new(),
                Address::ZERO,
            );

            let gas_limit = match call.estimate_gas().await {
                Ok(est) => est + est / 5,
                Err(_) => 3_000_000,
            };

            match call.gas(gas_limit).nonce(nonce).send().await {
                Ok(pending) => {
                    let tx_hash = format!("{}", pending.tx_hash());
                    info!(chain = %self.chain_name, nonce, tx = %tx_hash, "Morpho TX sent");

                    match pending.get_receipt().await {
                        Ok(receipt) => {
                            if receipt.status() {
                                self.stats.total_success += 1;
                                self.stats.total_profit_usd += candidate.estimated_profit_usd;
                                info!(chain = %self.chain_name, tx = %tx_hash, "MORPHO SUCCESS");
                                self.push_event("liquidation_success", &format!(
                                    "TX={} profit=${:.2} {}/{}",
                                    &tx_hash[..10], candidate.estimated_profit_usd,
                                    candidate.collateral_symbol, candidate.loan_symbol,
                                )).await;
                            } else {
                                self.stats.total_failed += 1;
                                error!(chain = %self.chain_name, tx = %tx_hash, "MORPHO REVERTED");
                                self.push_event("liquidation_failed", &format!("TX={} REVERTED", &tx_hash[..10])).await;
                            }
                        }
                        Err(e) => {
                            self.stats.total_failed += 1;
                            error!(chain = %self.chain_name, "Receipt failed: {e}");
                        }
                    }
                }
                Err(e) => {
                    self.stats.total_failed += 1;
                    error!(chain = %self.chain_name, "Morpho TX failed: {e}");
                    if e.to_string().contains("nonce") {
                        self.current_nonce = None;
                        break;
                    }
                }
            }
        }

        self.current_nonce = Some(base_nonce + executable.len() as u64);
    }

    async fn dry_run_execute(&mut self, candidate: &MorphoCandidate) {
        info!(
            chain = %self.chain_name,
            target = %candidate.borrower,
            hf = format!("{:.4}", candidate.health_factor),
            pair = format!("{} / {}", candidate.collateral_symbol, candidate.loan_symbol),
            borrowed = format!("{:.2}", candidate.borrowed_assets),
            profit = format!("${:.2}", candidate.estimated_profit_usd),
            "[DRY RUN] MORPHO LIQUIDATION CANDIDATE"
        );

        self.push_event("candidate_found", &format!(
            "HF={:.4} {} / {} borrowed={:.2} profit=${:.2}",
            candidate.health_factor,
            candidate.collateral_symbol,
            candidate.loan_symbol,
            candidate.borrowed_assets,
            candidate.estimated_profit_usd,
        )).await;
    }

    async fn live_execute<P: Provider + Clone>(
        &mut self,
        provider: &P,
        signer_address: Address,
        candidate: &MorphoCandidate,
    ) {
        let Some(contract_addr) = self.liquidator_contract else {
            error!(chain = %self.chain_name, "No Morpho liquidator contract configured");
            return;
        };

        let _t0 = Instant::now();

        info!(
            chain = %self.chain_name,
            target = %candidate.borrower,
            hf = format!("{:.4}", candidate.health_factor),
            pair = format!("{} / {}", candidate.collateral_symbol, candidate.loan_symbol),
            "MORPHO LIQUIDATION ATTEMPT"
        );

        let contract = IMorphoLiquidator::new(contract_addr, provider.clone());

        let mk_call = || {
            contract.liquidate(
                candidate.loan_token,
                candidate.collateral_token,
                candidate.oracle,
                candidate.irm,
                candidate.lltv,
                candidate.borrower,
                candidate.seized_assets,
                U256::ZERO,
                alloy::primitives::Bytes::new(),
                Address::ZERO,
            )
        };

        // Phase 1: Simulate
        match mk_call().from(signer_address).call().await {
            Ok(_) => {
                info!(chain = %self.chain_name, "Morpho sim OK");
            }
            Err(e) => {
                self.stats.total_sim_failed += 1;
                warn!(chain = %self.chain_name, "Morpho sim FAILED: {e}");
                self.push_event("sim_failed", &format!("Sim failed: {e}")).await;
                return;
            }
        }

        // Phase 2: Nonce
        let nonce = match self.current_nonce {
            Some(n) => n,
            None => match provider.get_transaction_count(signer_address).await {
                Ok(n) => { self.current_nonce = Some(n); n }
                Err(e) => {
                    error!(chain = %self.chain_name, "Nonce failed: {e}");
                    self.stats.total_failed += 1;
                    return;
                }
            },
        };

        // Phase 3: Send TX
        self.stats.total_attempted += 1;

        let send_call = mk_call();
        let gas_limit = match send_call.estimate_gas().await {
            Ok(est) => est + est / 5,
            Err(_) => 3_000_000,
        };

        match mk_call().gas(gas_limit).nonce(nonce).send().await {
            Ok(pending) => {
                let tx_hash = format!("{}", pending.tx_hash());
                info!(chain = %self.chain_name, tx = %tx_hash, "Morpho TX sent");
                self.current_nonce = Some(nonce + 1);

                match pending.get_receipt().await {
                    Ok(receipt) => {
                        if receipt.status() {
                            self.stats.total_success += 1;
                            self.stats.total_profit_usd += candidate.estimated_profit_usd;
                            info!(chain = %self.chain_name, tx = %tx_hash, "MORPHO SUCCESS");
                            self.push_event("liquidation_success", &format!(
                                "TX={} profit=${:.2} {}/{}",
                                &tx_hash[..10], candidate.estimated_profit_usd,
                                candidate.collateral_symbol, candidate.loan_symbol,
                            )).await;
                        } else {
                            self.stats.total_failed += 1;
                            error!(chain = %self.chain_name, tx = %tx_hash, "MORPHO REVERTED");
                            self.push_event("liquidation_failed", &format!("TX={} REVERTED", &tx_hash[..10])).await;
                        }
                    }
                    Err(e) => {
                        self.stats.total_failed += 1;
                        error!(chain = %self.chain_name, "Receipt failed: {e}");
                    }
                }
            }
            Err(e) => {
                self.stats.total_failed += 1;
                error!(chain = %self.chain_name, "Morpho TX failed: {e}");
                if e.to_string().contains("nonce") {
                    self.current_nonce = None;
                }
            }
        }
    }

    async fn push_event(&self, event_type: &str, message: &str) {
        let mut state = self.dashboard.write().await;
        state.push_event(DashboardEvent {
            timestamp_unix: now_unix(),
            chain: self.chain_name.clone(),
            event_type: event_type.to_string(),
            message: message.to_string(),
            data: None,
        });
    }

    pub fn clean_cooldowns(&mut self) {
        self.cooldowns.retain(|_, time| time.elapsed() < self.cooldown_duration);
    }

    pub fn stats(&self) -> &ExecutorStats {
        &self.stats
    }
}
