use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use alloy::sol_types::SolCall;
use eyre::Result;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use crate::core::config::ChainConfig;
use crate::core::multicall::{multicall_aggregate, MulticallRequest};
use crate::core::types::*;
use crate::protocols::aave_v3::contracts::{IAaveOracle, IPool, IPoolDataProvider};

/// Aave V3 position monitor — generic over any chain with Aave V3 deployment.
pub struct AaveV3Monitor {
    chain: ChainConfig,
    borrowers: HashSet<Address>,
    positions: HashMap<Address, Position>,
    asset_registry: HashMap<Address, AssetConfig>,
    e_modes: HashMap<u8, EModeConfig>,
    prices: HashMap<Address, U256>,
    all_asset_addresses: Vec<Address>,

    /// Pre-computed set of at-risk addresses (HF < 1.2) for quick scans
    at_risk_set: HashSet<Address>,

    // Stats
    scan_count: u64,
    last_scan_ms: u64,
    total_candidates_found: u64,
    last_scanned_block: u64,

    // Paths
    data_dir: PathBuf,
}

impl AaveV3Monitor {
    pub fn new(chain: ChainConfig) -> Self {
        let data_dir = PathBuf::from("data");
        Self {
            chain,
            borrowers: HashSet::new(),
            positions: HashMap::new(),
            asset_registry: HashMap::new(),
            e_modes: HashMap::new(),
            prices: HashMap::new(),
            all_asset_addresses: Vec::new(),
            at_risk_set: HashSet::new(),
            scan_count: 0,
            last_scan_ms: 0,
            total_candidates_found: 0,
            last_scanned_block: 0,
            data_dir,
        }
    }

    fn base_unit(&self) -> f64 {
        10f64.powi(self.chain.base_currency_decimals as i32)
    }

    /// Full initialization: load assets, discover borrowers, initial scan.
    pub async fn init<P: Provider + Clone>(&mut self, provider: &P) -> Result<()> {
        info!(
            chain = %self.chain.name,
            chain_id = self.chain.chain_id,
            "Initializing Aave V3 monitor"
        );

        fs::create_dir_all(&self.data_dir).ok();

        self.load_asset_registry(provider).await?;
        let _ = self.refresh_prices(provider).await?;
        self.load_borrower_cache();
        self.discover_borrowers(provider).await?;

        info!(
            chain = %self.chain.name,
            assets = self.asset_registry.len(),
            borrowers = self.borrowers.len(),
            "Init complete"
        );

        Ok(())
    }

    /// Load all reserve configurations from on-chain.
    async fn load_asset_registry<P: Provider + Clone>(&mut self, provider: &P) -> Result<()> {
        let dp = IPoolDataProvider::new(self.chain.aave_data_provider, provider.clone());
        let pool = IPool::new(self.chain.aave_pool, provider.clone());

        // Get all reserves
        // getAllReservesTokens returns Vec<TokenData> directly
        let reserve_tokens = dp.getAllReservesTokens().call().await?;
        info!(
            chain = %self.chain.name,
            count = reserve_tokens.len(),
            "Loading reserve tokens"
        );

        self.asset_registry.clear();
        self.all_asset_addresses.clear();

        for token_data in &reserve_tokens {
            let address = token_data.tokenAddress;
            let config_data = dp.getReserveConfigurationData(address).call().await?;

            // V2 forks don't have eMode — skip the call entirely
            let e_mode_cat = if self.chain.is_v2 {
                0u8
            } else {
                dp.getReserveEModeCategory(address).call().await
                    .map(|r| r.to::<u64>() as u8)
                    .unwrap_or(0)
            };

            let decimals = config_data.decimals.to::<u8>();
            let asset_config = AssetConfig {
                address,
                symbol: token_data.symbol.clone(),
                decimals,
                unit: U256::from(10u64).pow(U256::from(decimals)),
                liquidation_threshold: config_data.liquidationThreshold.to::<u64>(),
                liquidation_bonus: config_data.liquidationBonus.to::<u64>(),
                ltv: config_data.ltv.to::<u64>(),
                usage_as_collateral_enabled: config_data.usageAsCollateralEnabled,
                borrowing_enabled: config_data.borrowingEnabled,
                is_active: config_data.isActive,
                is_frozen: config_data.isFrozen,
                e_mode_category: e_mode_cat,
            };

            info!(
                "  {} LTV={}% LT={}% LB={}% eMode={}",
                asset_config.symbol,
                asset_config.ltv / 100,
                asset_config.liquidation_threshold / 100,
                asset_config.liquidation_bonus / 100,
                asset_config.e_mode_category,
            );

            self.all_asset_addresses.push(address);
            self.asset_registry.insert(address, asset_config);
        }

        // Load e-mode categories (1..10) — V2 forks don't have eMode
        if !self.chain.is_v2 {
            for id in 1u8..=10 {
                match pool.getEModeCategoryData(id).call().await {
                    Ok(data) if data.ltv > 0 => {
                        info!(
                            "  E-Mode {}: {} (LT={}, LB={}, LTV={})",
                            id, data.label, data.liquidationThreshold, data.liquidationBonus, data.ltv,
                        );
                        self.e_modes.insert(id, EModeConfig {
                            category_id: id,
                            label: data.label,
                            liquidation_threshold: data.liquidationThreshold as u64,
                            liquidation_bonus: data.liquidationBonus as u64,
                            ltv: data.ltv as u64,
                            price_source: data.priceSource,
                        });
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Refresh oracle prices for all assets.
    /// Returns list of assets whose prices changed (for targeted re-evaluation).
    pub async fn refresh_prices<P: Provider + Clone>(&mut self, provider: &P) -> Result<Vec<Address>> {
        if self.all_asset_addresses.is_empty() {
            return Ok(vec![]);
        }

        let oracle = IAaveOracle::new(self.chain.aave_oracle, provider.clone());
        let prices = oracle
            .getAssetsPrices(self.all_asset_addresses.clone())
            .call()
            .await?;

        let mut changed_assets = Vec::new();
        for (i, addr) in self.all_asset_addresses.iter().enumerate() {
            let price = prices[i];
            if price > U256::ZERO {
                let old = self.prices.get(addr);
                if old != Some(&price) {
                    // Calculate price change percentage for logging
                    if let Some(old_price) = old {
                        let old_f = u256_to_f64(*old_price);
                        let new_f = u256_to_f64(price);
                        if old_f > 0.0 {
                            let pct_change = ((new_f - old_f) / old_f * 100.0).abs();
                            if pct_change > 0.1 {
                                let symbol = self.get_symbol(*addr);
                                debug!(
                                    chain = %self.chain.name,
                                    asset = %symbol,
                                    change = format!("{:.2}%", if new_f > old_f { pct_change } else { -pct_change }),
                                    "Price changed"
                                );
                            }
                        }
                    }
                    self.prices.insert(*addr, price);
                    changed_assets.push(*addr);
                }
            }
        }

        if !changed_assets.is_empty() {
            debug!(
                chain = %self.chain.name,
                updated = changed_assets.len(),
                total = self.all_asset_addresses.len(),
                "Prices refreshed"
            );
        }

        Ok(changed_assets)
    }

    /// Compute the correct Borrow event topic for V2 vs V3.
    /// V3: Borrow(address,address,address,uint256,uint8,uint256,uint16) — interestRateMode=uint8
    /// V2: Borrow(address,address,address,uint256,uint256,uint256,uint16) — borrowRateMode=uint256
    fn borrow_event_topic(&self) -> FixedBytes<32> {
        if self.chain.is_v2 {
            alloy::primitives::keccak256(
                b"Borrow(address,address,address,uint256,uint256,uint256,uint16)",
            )
        } else {
            alloy::primitives::keccak256(
                b"Borrow(address,address,address,uint256,uint8,uint256,uint16)",
            )
        }
    }

    /// Scan Borrow events to discover all borrower addresses.
    async fn discover_borrowers<P: Provider + Clone>(&mut self, provider: &P) -> Result<()> {
        let current_block = provider.get_block_number().await?;
        let start_block = if self.last_scanned_block > 0 {
            self.last_scanned_block + 1
        } else {
            self.chain.borrow_start_block
        };

        if start_block >= current_block {
            return Ok(());
        }

        let borrow_topic = self.borrow_event_topic();

        let chunk_size = self.chain.get_logs_chunk_size;
        let total_blocks = current_block - start_block;
        let prev_count = self.borrowers.len();

        info!(
            chain = %self.chain.name,
            from = start_block,
            to = current_block,
            blocks = total_blocks,
            "Scanning Borrow events"
        );

        let mut from = start_block;
        while from <= current_block {
            let to = (from + chunk_size - 1).min(current_block);

            let filter = Filter::new()
                .address(self.chain.aave_pool)
                .event_signature(borrow_topic)
                .from_block(from)
                .to_block(to);

            match provider.get_logs(&filter).await {
                Ok(logs) => {
                    for log in &logs {
                        // topic[2] = user (indexed)
                        if let Some(topic) = log.topics().get(2) {
                            let borrower = Address::from_slice(&topic[12..]);
                            self.borrowers.insert(borrower);
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        chain = %self.chain.name,
                        from, to,
                        "getLogs failed, retrying with half chunk: {e}"
                    );
                    // Retry with half chunk
                    let half = chunk_size / 2;
                    let mut sub_from = from;
                    while sub_from <= to {
                        let sub_to = (sub_from + half - 1).min(to);
                        let sub_filter = Filter::new()
                            .address(self.chain.aave_pool)
                            .event_signature(borrow_topic)
                            .from_block(sub_from)
                            .to_block(sub_to);

                        match provider.get_logs(&sub_filter).await {
                            Ok(logs) => {
                                for log in &logs {
                                    if let Some(topic) = log.topics().get(2) {
                                        let borrower = Address::from_slice(&topic[12..]);
                                        self.borrowers.insert(borrower);
                                    }
                                }
                            }
                            Err(e) => {
                                error!(
                                    chain = %self.chain.name,
                                    from = sub_from, to = sub_to,
                                    "Retry also failed: {e}"
                                );
                            }
                        }
                        sub_from = sub_to + 1;
                    }
                }
            }

            // Rate limit delay — longer for chains with aggressive limits (dRPC free tier)
            let delay_ms = if chunk_size <= 1_000 { 500 } else { 200 };
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;

            // Progress logging every ~100K blocks
            if from > start_block && (from - start_block) % 100_000 < chunk_size {
                let pct = ((from - start_block) as f64 / total_blocks as f64 * 100.0) as u32;
                info!(
                    chain = %self.chain.name,
                    progress = %format!("{pct}%"),
                    borrowers = self.borrowers.len(),
                    "Scan progress"
                );
            }

            from = to + 1;
        }

        self.last_scanned_block = current_block;
        let new_borrowers = self.borrowers.len() - prev_count;
        info!(
            chain = %self.chain.name,
            total = self.borrowers.len(),
            new = new_borrowers,
            "Borrower discovery complete"
        );

        self.save_borrower_cache();
        Ok(())
    }

    /// Full scan: batch getUserAccountData for ALL borrowers, find candidates.
    pub async fn full_scan<P: Provider + Clone>(
        &mut self,
        provider: &P,
    ) -> Result<Vec<LiquidationCandidate>> {
        let start = std::time::Instant::now();
        self.scan_count += 1;

        let borrower_list: Vec<Address> = self.borrowers.iter().copied().collect();
        if borrower_list.is_empty() {
            warn!(chain = %self.chain.name, "No borrowers to scan");
            return Ok(vec![]);
        }

        info!(
            chain = %self.chain.name,
            scan = self.scan_count,
            borrowers = borrower_list.len(),
            "Full scan"
        );

        let _changed_prices = self.refresh_prices(provider).await?;
        let current_block = provider.get_block_number().await?;

        // Build multicall requests for getUserAccountData
        let requests: Vec<MulticallRequest> = borrower_list
            .iter()
            .map(|user| {
                let call_data = IPool::getUserAccountDataCall { user: *user }.abi_encode();
                MulticallRequest {
                    target: self.chain.aave_pool,
                    call_data: Bytes::from(call_data),
                }
            })
            .collect();

        let results = multicall_aggregate(provider, self.chain.multicall3, &requests, 500).await?;

        let min_debt_base = U256::from((self.chain.min_debt_usd * self.base_unit()) as u64);
        let hf_one = U256::from(HF_ONE);
        let hf_1_2 = U256::from(HF_ONE) * U256::from(12) / U256::from(10);

        let mut active_positions = 0u32;
        let mut dust_skipped = 0u32;
        let mut no_debt_skipped = 0u32;
        let mut at_risk_addresses: Vec<Address> = Vec::new();
        let mut candidates: Vec<LiquidationCandidate> = Vec::new();

        for (i, result) in results.iter().enumerate() {
            if !result.success || result.return_data.is_empty() {
                continue;
            }

            // Decode getUserAccountData return
            let decoded = match IPool::getUserAccountDataCall::abi_decode_returns(
                &result.return_data,
            ) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let user_addr = borrower_list[i];
            let total_debt_base = decoded.totalDebtBase;
            let total_collateral_base = decoded.totalCollateralBase;
            let health_factor = decoded.healthFactor;

            // Skip zero debt
            if total_debt_base.is_zero() {
                no_debt_skipped += 1;
                self.positions.remove(&user_addr);
                continue;
            }

            // Skip dust
            if total_debt_base < min_debt_base {
                dust_skipped += 1;
                continue;
            }

            active_positions += 1;

            // Update position
            let existing = self.positions.get(&user_addr);
            let position = Position {
                address: user_addr,
                total_collateral_base,
                total_debt_base,
                avg_liquidation_threshold: decoded.currentLiquidationThreshold,
                health_factor,
                reserves: existing.map(|p| p.reserves.clone()).unwrap_or_default(),
                e_mode_category: existing.map(|p| p.e_mode_category).unwrap_or(0),
                last_update_block: current_block,
            };
            self.positions.insert(user_addr, position);

            // Track at-risk (HF < 1.2)
            if health_factor < hf_1_2 {
                at_risk_addresses.push(user_addr);
            }

            // Liquidatable (HF < 1.0)
            if health_factor < hf_one && !health_factor.is_zero() {
                candidates.push(self.build_candidate_from_aggregate(user_addr));
            }
        }

        // Load detailed reserves for at-risk positions
        if !at_risk_addresses.is_empty() {
            self.load_detailed_reserves(provider, &at_risk_addresses, current_block)
                .await?;
            self.load_e_modes(provider, &at_risk_addresses).await?;

            // Re-evaluate with detailed data
            candidates.clear();
            for addr in &at_risk_addresses {
                if let Some(pos) = self.positions.get(addr) {
                    if pos.health_factor < hf_one && !pos.health_factor.is_zero() {
                        if let Some(c) = self.build_candidate_from_detailed(*addr) {
                            candidates.push(c);
                        }
                    }
                }
            }
        }

        self.last_scan_ms = start.elapsed().as_millis() as u64;
        self.total_candidates_found += candidates.len() as u64;

        // Update at-risk set for quick scans
        self.at_risk_set = self
            .positions
            .iter()
            .filter(|(_, p)| p.health_factor < hf_1_2 && !p.health_factor.is_zero())
            .map(|(addr, _)| *addr)
            .collect();

        let at_risk_count = self.at_risk_set.len();

        info!(
            chain = %self.chain.name,
            scan = self.scan_count,
            ms = self.last_scan_ms,
            active = active_positions,
            at_risk = at_risk_count,
            liquidatable = candidates.len(),
            dust = dust_skipped,
            no_debt = no_debt_skipped,
            "Scan complete"
        );

        // Log candidates
        for c in &candidates {
            info!(
                chain = %self.chain.name,
                addr = %format!("{}...", &format!("{}", c.address)[..10]),
                hf = format!("{:.4}", c.health_factor),
                debt_usd = format!("${:.0}", c.total_debt_usd),
                col_usd = format!("${:.0}", c.total_collateral_usd),
                pair = format!("{}->{}", c.debt_symbol, c.collateral_symbol),
                profit = format!("${:.2}", c.estimated_profit_usd),
                "CANDIDATE"
            );
        }

        // Log top at-risk if no candidates
        if at_risk_addresses.len() > 0 && candidates.is_empty() {
            let mut near_liq: Vec<&Position> = at_risk_addresses
                .iter()
                .filter_map(|a| self.positions.get(a))
                .collect();
            near_liq.sort_by(|a, b| a.health_factor.cmp(&b.health_factor));

            for pos in near_liq.iter().take(10) {
                let hf = u256_to_f64(pos.health_factor) / 1e18;
                let debt_usd = u256_to_f64(pos.total_debt_base) / self.base_unit();
                info!(
                    chain = %self.chain.name,
                    addr = %format!("{}...", &format!("{}", pos.address)[..10]),
                    hf = format!("{:.4}", hf),
                    debt_usd = format!("${:.0}", debt_usd),
                    "AT-RISK"
                );
            }
        }

        Ok(candidates)
    }

    /// Load per-reserve data for specific users via Multicall3.
    async fn load_detailed_reserves<P: Provider + Clone>(
        &mut self,
        provider: &P,
        users: &[Address],
        block_number: u64,
    ) -> Result<()> {
        if users.is_empty() || self.all_asset_addresses.is_empty() {
            return Ok(());
        }

        let mut requests = Vec::new();
        let mut request_index: Vec<(usize, usize)> = Vec::new();

        for (u, user) in users.iter().enumerate() {
            for (t, token) in self.all_asset_addresses.iter().enumerate() {
                let call_data = IPoolDataProvider::getUserReserveDataCall {
                    asset: *token,
                    user: *user,
                }
                .abi_encode();
                requests.push(MulticallRequest {
                    target: self.chain.aave_data_provider,
                    call_data: Bytes::from(call_data),
                });
                request_index.push((u, t));
            }
        }

        let results =
            multicall_aggregate(provider, self.chain.multicall3, &requests, 500).await?;

        let mut reserve_map: HashMap<Address, Vec<UserReserveData>> = HashMap::new();
        for user in users {
            reserve_map.insert(*user, Vec::new());
        }

        for (i, result) in results.iter().enumerate() {
            if !result.success || result.return_data.is_empty() {
                continue;
            }

            let (user_idx, token_idx) = request_index[i];
            let user_addr = users[user_idx];
            let token_addr = self.all_asset_addresses[token_idx];

            let decoded = match IPoolDataProvider::getUserReserveDataCall::abi_decode_returns(
                &result.return_data,
            ) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let a_token_balance = decoded.currentATokenBalance;
            let stable_debt = decoded.currentStableDebt;
            let variable_debt = decoded.currentVariableDebt;

            if a_token_balance.is_zero() && stable_debt.is_zero() && variable_debt.is_zero() {
                continue;
            }

            if let Some(reserves) = reserve_map.get_mut(&user_addr) {
                reserves.push(UserReserveData {
                    asset: token_addr,
                    a_token_balance,
                    stable_debt,
                    variable_debt,
                    usage_as_collateral: decoded.usageAsCollateralEnabled,
                });
            }
        }

        for (addr, reserves) in reserve_map {
            if let Some(pos) = self.positions.get_mut(&addr) {
                if !reserves.is_empty() {
                    pos.reserves = reserves;
                    pos.last_update_block = block_number;
                }
            }
        }

        info!(
            chain = %self.chain.name,
            users = users.len(),
            "Loaded detailed reserves"
        );
        Ok(())
    }

    /// Load e-mode categories for specific users. Skipped for V2 forks.
    async fn load_e_modes<P: Provider + Clone>(
        &mut self,
        provider: &P,
        users: &[Address],
    ) -> Result<()> {
        if self.chain.is_v2 {
            return Ok(());
        }

        let requests: Vec<MulticallRequest> = users
            .iter()
            .map(|user| {
                let call_data = IPool::getUserEModeCall { user: *user }.abi_encode();
                MulticallRequest {
                    target: self.chain.aave_pool,
                    call_data: Bytes::from(call_data),
                }
            })
            .collect();

        let results =
            multicall_aggregate(provider, self.chain.multicall3, &requests, 500).await?;

        for (i, result) in results.iter().enumerate() {
            if !result.success || result.return_data.is_empty() {
                continue;
            }
            if let Ok(decoded) =
                IPool::getUserEModeCall::abi_decode_returns(&result.return_data)
            {
                // getUserEMode returns a single uint256
                let e_mode_id = decoded.to::<u64>() as u8;
                if let Some(pos) = self.positions.get_mut(&users[i]) {
                    pos.e_mode_category = e_mode_id;
                }
            }
        }

        Ok(())
    }

    // ── Quick Scan (block-reactive, at-risk only) ──

    /// Fast scan: only re-check at-risk positions + recently updated prices.
    /// Called on every new block. Much faster than full_scan.
    pub async fn quick_scan<P: Provider + Clone>(
        &mut self,
        provider: &P,
    ) -> Result<Vec<LiquidationCandidate>> {
        if self.at_risk_set.is_empty() {
            return Ok(vec![]);
        }

        let start = std::time::Instant::now();

        // Refresh prices (fast — single multicall)
        let _changed = self.refresh_prices(provider).await?;

        let at_risk_list: Vec<Address> = self.at_risk_set.iter().copied().collect();

        // Batch getUserAccountData only for at-risk positions
        let requests: Vec<MulticallRequest> = at_risk_list
            .iter()
            .map(|user| {
                let call_data = IPool::getUserAccountDataCall { user: *user }.abi_encode();
                MulticallRequest {
                    target: self.chain.aave_pool,
                    call_data: Bytes::from(call_data),
                }
            })
            .collect();

        let results = multicall_aggregate(provider, self.chain.multicall3, &requests, 500).await?;

        let hf_one = U256::from(HF_ONE);
        let hf_1_2 = U256::from(HF_ONE) * U256::from(12) / U256::from(10);
        let current_block = provider.get_block_number().await?;
        let mut candidates: Vec<LiquidationCandidate> = Vec::new();
        let mut new_at_risk = HashSet::new();

        for (i, result) in results.iter().enumerate() {
            if !result.success || result.return_data.is_empty() {
                continue;
            }

            let decoded = match IPool::getUserAccountDataCall::abi_decode_returns(
                &result.return_data,
            ) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let user_addr = at_risk_list[i];
            let health_factor = decoded.healthFactor;
            let total_debt_base = decoded.totalDebtBase;

            if total_debt_base.is_zero() {
                self.positions.remove(&user_addr);
                continue;
            }

            // Update position
            if let Some(pos) = self.positions.get_mut(&user_addr) {
                pos.health_factor = health_factor;
                pos.total_collateral_base = decoded.totalCollateralBase;
                pos.total_debt_base = total_debt_base;
                pos.avg_liquidation_threshold = decoded.currentLiquidationThreshold;
                pos.last_update_block = current_block;
            }

            // Still at risk?
            if health_factor < hf_1_2 && !health_factor.is_zero() {
                new_at_risk.insert(user_addr);
            }

            // Liquidatable?
            if health_factor < hf_one && !health_factor.is_zero() {
                // Load detailed reserves for liquidatable positions
                self.load_detailed_reserves(provider, &[user_addr], current_block).await?;
                self.load_e_modes(provider, &[user_addr]).await?;

                if let Some(c) = self.build_candidate_from_detailed(user_addr) {
                    info!(
                        chain = %self.chain.name,
                        addr = %format!("{}...", &format!("{}", c.address)[..10]),
                        hf = format!("{:.4}", c.health_factor),
                        profit = format!("${:.2}", c.estimated_profit_usd),
                        "QUICK SCAN: CANDIDATE FOUND"
                    );
                    candidates.push(c);
                }
            }
        }

        self.at_risk_set = new_at_risk;
        self.total_candidates_found += candidates.len() as u64;

        let elapsed_ms = start.elapsed().as_millis() as u64;
        if elapsed_ms > 100 || !candidates.is_empty() {
            debug!(
                chain = %self.chain.name,
                ms = elapsed_ms,
                checked = at_risk_list.len(),
                candidates = candidates.len(),
                "Quick scan"
            );
        }

        Ok(candidates)
    }

    /// Incremental borrower discovery: check for new Borrow events since last scanned block.
    /// Called periodically (every ~30s) to catch new borrowers without full re-scan.
    pub async fn incremental_discover<P: Provider + Clone>(
        &mut self,
        provider: &P,
        current_block: u64,
    ) -> Result<u32> {
        // Leave a small buffer — some RPCs can't serve getLogs on the very latest blocks
        let safe_block = current_block.saturating_sub(2);
        let start_block = self.last_scanned_block + 1;
        if start_block >= safe_block {
            return Ok(0);
        }

        // Cap range to avoid querying too many blocks at once
        let max_range = self.chain.get_logs_chunk_size;
        let end_block = safe_block.min(start_block + max_range - 1);

        let borrow_topic = self.borrow_event_topic();

        let filter = Filter::new()
            .address(self.chain.aave_pool)
            .event_signature(borrow_topic)
            .from_block(start_block)
            .to_block(end_block);

        let mut new_count = 0u32;
        match provider.get_logs(&filter).await {
            Ok(logs) => {
                for log in &logs {
                    if let Some(topic) = log.topics().get(2) {
                        let borrower = Address::from_slice(&topic[12..]);
                        if self.borrowers.insert(borrower) {
                            new_count += 1;
                        }
                    }
                }
            }
            Err(e) => {
                warn!(
                    chain = %self.chain.name,
                    "Incremental discover failed: {e}"
                );
                return Ok(0);
            }
        }

        self.last_scanned_block = end_block;

        if new_count > 0 {
            info!(
                chain = %self.chain.name,
                new = new_count,
                total = self.borrowers.len(),
                "New borrowers discovered"
            );
            self.save_borrower_cache();
        }

        Ok(new_count)
    }

    /// Get the set of at-risk addresses for external consumers
    pub fn at_risk_count(&self) -> usize {
        self.at_risk_set.len()
    }

    /// Pre-compute price drop thresholds for at-risk positions.
    /// Returns: Vec of (user, asset_that_would_trigger, pct_drop_needed)
    /// This tells us: "if asset X drops by Y%, user Z becomes liquidatable"
    pub fn compute_liquidation_thresholds(&self) -> Vec<(Address, String, f64)> {
        let hf_one_f = HF_ONE as f64;
        let mut thresholds = Vec::new();

        for addr in &self.at_risk_set {
            let Some(pos) = self.positions.get(addr) else {
                continue;
            };

            let hf = u256_to_f64(pos.health_factor);
            if hf <= 0.0 || hf >= 1.2 * hf_one_f {
                continue;
            }

            // HF = (collateral_base * avg_LT / 10000) / debt_base
            // For liquidation: HF < 1.0
            // If collateral drops by X%: new_HF = HF * (1 - X/100)
            // We need: HF * (1 - X/100) < 1.0
            // So: X/100 > 1 - 1.0/HF_normalized
            // X = (1 - 1e18/HF) * 100

            let hf_normalized = hf / hf_one_f;
            if hf_normalized <= 0.0 {
                continue;
            }

            let pct_drop = (1.0 - 1.0 / hf_normalized) * 100.0;

            if pct_drop > 0.0 && pct_drop < 50.0 {
                // Find the dominant collateral asset
                let collateral_symbol = if pos.reserves.is_empty() {
                    "unknown".to_string()
                } else {
                    // Find largest collateral
                    pos.reserves
                        .iter()
                        .filter(|r| r.usage_as_collateral && !r.a_token_balance.is_zero())
                        .max_by_key(|r| {
                            let price = self.prices.get(&r.asset).copied().unwrap_or(U256::ZERO);
                            let config = self.asset_registry.get(&r.asset);
                            let unit = config.map(|c| c.unit).unwrap_or(U256::from(1u64));
                            r.a_token_balance * price / unit
                        })
                        .map(|r| self.get_symbol(r.asset))
                        .unwrap_or_else(|| "unknown".to_string())
                };

                thresholds.push((*addr, collateral_symbol, pct_drop));
            }
        }

        // Sort by smallest drop needed (most at-risk first)
        thresholds.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        thresholds
    }

    // ── Candidate Building ──

    fn build_candidate_from_aggregate(&self, addr: Address) -> LiquidationCandidate {
        let pos = self.positions.get(&addr).unwrap();
        let hf = u256_to_f64(pos.health_factor) / 1e18;
        let debt_usd = u256_to_f64(pos.total_debt_base) / self.base_unit();
        let col_usd = u256_to_f64(pos.total_collateral_base) / self.base_unit();

        // V2 forks (Lendle) always have 50% close factor.
        // V3 has dynamic close factor: 100% when HF < 0.95.
        let close_factor = if self.chain.is_v2 {
            U256::from(BPS_PRECISION / 2)
        } else if hf < 0.95 {
            U256::from(BPS_PRECISION)
        } else {
            U256::from(BPS_PRECISION / 2)
        };

        LiquidationCandidate {
            address: addr,
            health_factor: hf,
            total_collateral_usd: col_usd,
            total_debt_usd: debt_usd,
            collateral_asset: Address::ZERO,
            collateral_symbol: "??".to_string(),
            collateral_decimals: 18,
            collateral_price: U256::ZERO,
            debt_asset: Address::ZERO,
            debt_symbol: "??".to_string(),
            debt_decimals: 18,
            debt_price: U256::ZERO,
            debt_to_cover: U256::ZERO,
            estimated_profit_usd: 0.0,
            liquidation_bonus: 0,
            close_factor,
            e_mode_category: pos.e_mode_category,
        }
    }

    fn build_candidate_from_detailed(&self, addr: Address) -> Option<LiquidationCandidate> {
        let pos = self.positions.get(&addr)?;
        if pos.reserves.is_empty() {
            return Some(self.build_candidate_from_aggregate(addr));
        }

        let mut best_collateral: Option<(Address, U256, u64, U256)> = None; // (asset, value, bonus, balance)
        let mut best_debt: Option<(Address, U256, U256)> = None; // (asset, value, totalDebt)

        for reserve in &pos.reserves {
            let config = self.asset_registry.get(&reserve.asset)?;
            let price = self.prices.get(&reserve.asset)?;
            if price.is_zero() {
                continue;
            }

            // Collateral
            if reserve.usage_as_collateral && !reserve.a_token_balance.is_zero() {
                let value = reserve.a_token_balance * *price / config.unit;

                // Get bonus (e-mode or standard)
                let bonus = if pos.e_mode_category > 0 {
                    self.e_modes
                        .get(&pos.e_mode_category)
                        .filter(|_| config.e_mode_category == pos.e_mode_category)
                        .map(|em| em.liquidation_bonus)
                        .unwrap_or(config.liquidation_bonus)
                } else {
                    config.liquidation_bonus
                };

                if bonus > 0 {
                    if best_collateral.is_none() || value > best_collateral.as_ref().unwrap().1 {
                        best_collateral =
                            Some((reserve.asset, value, bonus, reserve.a_token_balance));
                    }
                }
            }

            // Debt
            let total_debt = reserve.variable_debt + reserve.stable_debt;
            if !total_debt.is_zero() {
                let value = total_debt * *price / config.unit;
                if best_debt.is_none() || value > best_debt.as_ref().unwrap().1 {
                    best_debt = Some((reserve.asset, value, total_debt));
                }
            }
        }

        let (col_asset, _col_value, col_bonus, _col_balance) = best_collateral?;
        let (debt_asset, _debt_value, debt_total) = best_debt?;

        let debt_config = self.asset_registry.get(&debt_asset)?;
        let col_config = self.asset_registry.get(&col_asset)?;

        let hf = u256_to_f64(pos.health_factor) / 1e18;
        // V2 forks (Lendle) always have 50% close factor.
        // V3 has dynamic close factor: 100% when HF < 0.95.
        let close_factor = if self.chain.is_v2 {
            U256::from(BPS_PRECISION / 2)
        } else if hf < 0.95 {
            U256::from(BPS_PRECISION)
        } else {
            U256::from(BPS_PRECISION / 2)
        };
        let debt_to_cover = debt_total * close_factor / U256::from(BPS_PRECISION);

        let debt_price = self.prices.get(&debt_asset)?;
        if debt_price.is_zero() {
            return Some(self.build_candidate_from_aggregate(addr));
        }

        let debt_value_base = debt_to_cover * *debt_price / debt_config.unit;
        let bonus_part = col_bonus.saturating_sub(BPS_PRECISION as u64);
        let gross_profit_base =
            debt_value_base * U256::from(bonus_part) / U256::from(BPS_PRECISION);

        let profit_usd = u256_to_f64(gross_profit_base) / self.base_unit();
        let debt_usd = u256_to_f64(pos.total_debt_base) / self.base_unit();
        let col_usd = u256_to_f64(pos.total_collateral_base) / self.base_unit();

        let debt_symbol = self.get_symbol(debt_asset);
        let col_symbol = self.get_symbol(col_asset);

        Some(LiquidationCandidate {
            address: addr,
            health_factor: hf,
            total_collateral_usd: col_usd,
            total_debt_usd: debt_usd,
            collateral_asset: col_asset,
            collateral_symbol: col_symbol,
            collateral_decimals: col_config.decimals,
            collateral_price: *self.prices.get(&col_asset).unwrap_or(&U256::ZERO),
            debt_asset,
            debt_symbol,
            debt_decimals: debt_config.decimals,
            debt_price: *debt_price,
            debt_to_cover,
            estimated_profit_usd: profit_usd,
            liquidation_bonus: col_bonus,
            close_factor,
            e_mode_category: pos.e_mode_category,
        })
    }

    fn get_symbol(&self, asset: Address) -> String {
        if let Some(config) = self.asset_registry.get(&asset) {
            return config.symbol.clone();
        }
        if let Some(token) = self.chain.tokens.get(&asset) {
            return token.symbol.clone();
        }
        format!("{}", &format!("{asset}")[..8])
    }

    // ── Borrower Cache ──

    fn cache_path(&self) -> PathBuf {
        self.data_dir
            .join(format!("{}-borrowers.json", self.chain.name.to_lowercase()))
    }

    fn load_borrower_cache(&mut self) {
        let path = self.cache_path();
        let Ok(data) = fs::read_to_string(&path) else {
            return;
        };

        #[derive(serde::Deserialize)]
        struct Cache {
            borrowers: Vec<String>,
            #[serde(default)]
            last_block: u64,
        }

        let Ok(cache) = serde_json::from_str::<Cache>(&data) else {
            return;
        };

        let mut added = 0u32;
        for addr_str in &cache.borrowers {
            if let Ok(addr) = addr_str.parse::<Address>() {
                if self.borrowers.insert(addr) {
                    added += 1;
                }
            }
        }

        if cache.last_block > 0 {
            self.last_scanned_block = cache.last_block;
        }

        if !cache.borrowers.is_empty() {
            info!(
                chain = %self.chain.name,
                loaded = cache.borrowers.len(),
                new = added,
                last_block = self.last_scanned_block,
                "Loaded borrower cache"
            );
        }
    }

    fn save_borrower_cache(&self) {
        let path = self.cache_path();

        let borrowers: Vec<String> = self.borrowers.iter().map(|a| format!("{a}")).collect();

        let cache = serde_json::json!({
            "chain": self.chain.name,
            "chainId": self.chain.chain_id,
            "savedAt": chrono_now(),
            "count": self.borrowers.len(),
            "last_block": self.last_scanned_block,
            "borrowers": borrowers,
        });

        let tmp = path.with_extension("json.tmp");
        if let Ok(()) = fs::write(&tmp, serde_json::to_string(&cache).unwrap_or_default()) {
            fs::rename(&tmp, &path).ok();
            info!(
                chain = %self.chain.name,
                count = self.borrowers.len(),
                "Borrower cache saved"
            );
        }
    }

    // ── Getters ──

    pub fn borrower_count(&self) -> usize {
        self.borrowers.len()
    }

    pub fn position_count(&self) -> usize {
        self.positions.len()
    }

    pub fn last_scan_duration_ms(&self) -> u64 {
        self.last_scan_ms
    }

    pub fn total_scans(&self) -> u64 {
        self.scan_count
    }

    #[allow(dead_code)]
    pub fn chain_name(&self) -> &str {
        &self.chain.name
    }

    #[allow(dead_code)]
    pub fn scan_interval_ms(&self) -> u64 {
        self.chain.scan_interval_ms
    }
}

fn u256_to_f64(v: U256) -> f64 {
    let s = v.to_string();
    s.parse::<f64>().unwrap_or(0.0)
}

fn chrono_now() -> String {
    // Simple UTC timestamp without chrono dependency
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}s", d.as_secs())
}
