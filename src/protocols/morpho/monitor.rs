use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use alloy::sol_types::SolCall;
use eyre::Result;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::core::multicall::{multicall_aggregate, MulticallRequest};
use crate::protocols::morpho::contracts::{IMorpho, IMorphoOracle};

/// ORACLE_PRICE_SCALE = 1e36 (Morpho oracle prices are scaled by this)
const ORACLE_PRICE_SCALE: u128 = 1_000_000_000_000_000_000_000_000_000_000_000_000;
/// WAD = 1e18 (LLTV is in WAD)
const WAD: u128 = 1_000_000_000_000_000_000;

/// Morpho market parameters (from on-chain)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MorphoMarket {
    pub id: FixedBytes<32>,
    pub loan_token: Address,
    pub collateral_token: Address,
    pub oracle: Address,
    pub irm: Address,
    pub lltv: U256,
    // Cached market state
    pub total_borrow_assets: u128,
    pub total_borrow_shares: u128,
    pub total_supply_assets: u128,
    // Cached oracle price
    pub oracle_price: U256,
    // Token symbols (from chain config)
    pub loan_symbol: String,
    pub collateral_symbol: String,
    pub loan_decimals: u8,
    pub collateral_decimals: u8,
}

/// A borrower position in a Morpho market
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MorphoPosition {
    pub market_id: FixedBytes<32>,
    pub borrower: Address,
    pub borrow_shares: u128,
    pub collateral: u128,
    pub borrowed_assets: f64,
    pub collateral_value_usd: f64,
    pub health_factor: f64,
}

/// Morpho Blue liquidation candidate
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MorphoCandidate {
    pub market_id: FixedBytes<32>,
    pub borrower: Address,
    pub health_factor: f64,
    pub borrowed_assets: f64,
    pub collateral_amount: u128,
    pub seized_assets: U256,
    pub estimated_profit_usd: f64,
    pub loan_token: Address,
    pub collateral_token: Address,
    pub loan_symbol: String,
    pub collateral_symbol: String,
    pub oracle: Address,
    pub irm: Address,
    pub lltv: U256,
}

/// Morpho Blue monitor configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MorphoConfig {
    pub morpho_address: Address,
    pub multicall3: Address,
    pub chain_name: String,
    pub start_block: u64,
    pub get_logs_chunk_size: u64,
    pub scan_interval_ms: u64,
    pub poll_interval_ms: u64,
    pub min_debt_usd: f64,
    pub min_profit_usd: f64,
    /// Token address -> (symbol, decimals)
    pub tokens: HashMap<Address, (String, u8)>,
}

/// Morpho Blue position monitor for HyperEVM
pub struct MorphoMonitor {
    config: MorphoConfig,
    markets: HashMap<FixedBytes<32>, MorphoMarket>,
    borrowers_by_market: HashMap<FixedBytes<32>, HashSet<Address>>,
    positions: HashMap<(FixedBytes<32>, Address), MorphoPosition>,
    at_risk_set: HashSet<(FixedBytes<32>, Address)>,
    last_scanned_block: u64,
    scan_count: u64,
    last_scan_ms: u64,
    total_candidates_found: u64,
    data_dir: PathBuf,
}

impl MorphoMonitor {
    pub fn new(config: MorphoConfig) -> Self {
        Self {
            config,
            markets: HashMap::new(),
            borrowers_by_market: HashMap::new(),
            positions: HashMap::new(),
            at_risk_set: HashSet::new(),
            last_scanned_block: 0,
            scan_count: 0,
            last_scan_ms: 0,
            total_candidates_found: 0,
            data_dir: PathBuf::from("data"),
        }
    }

    pub async fn init<P: Provider + Clone>(&mut self, provider: &P) -> Result<()> {
        info!(chain = %self.config.chain_name, "Initializing Morpho Blue monitor");
        fs::create_dir_all(&self.data_dir).ok();
        self.load_cache();
        self.discover_markets(provider).await?;
        self.discover_borrowers(provider).await?;
        info!(
            chain = %self.config.chain_name,
            markets = self.markets.len(),
            borrowers = self.total_borrowers(),
            "Morpho init complete"
        );
        Ok(())
    }

    /// Discover all markets via CreateMarket events
    async fn discover_markets<P: Provider + Clone>(&mut self, provider: &P) -> Result<()> {
        let create_market_topic = alloy::primitives::keccak256(
            b"CreateMarket(bytes32,(address,address,address,address,uint256))",
        );

        let current_block = provider.get_block_number().await?;
        let chunk = self.config.get_logs_chunk_size;
        let mut from = self.config.start_block;

        info!(
            chain = %self.config.chain_name,
            from, to = current_block,
            "Scanning CreateMarket events"
        );

        while from <= current_block {
            let to = (from + chunk - 1).min(current_block);

            let filter = Filter::new()
                .address(self.config.morpho_address)
                .event_signature(create_market_topic)
                .from_block(from)
                .to_block(to);

            match provider.get_logs(&filter).await {
                Ok(logs) => {
                    for log in &logs {
                        if let Some(market_id) = log.topics().get(1) {
                            self.parse_create_market_log(*market_id, &log.data().data);
                        }
                    }
                }
                Err(e) => {
                    warn!(chain = %self.config.chain_name, from, to, "CreateMarket getLogs error: {e}");
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            from = to + 1;
        }

        info!(
            chain = %self.config.chain_name,
            markets = self.markets.len(),
            "Markets discovered"
        );

        for m in self.markets.values() {
            info!(
                "  Market {}: {} / {} (LLTV={:.1}%)",
                &format!("{}", m.id)[..10],
                m.collateral_symbol,
                m.loan_symbol,
                u256_to_f64(m.lltv) / 1e18 * 100.0,
            );
        }

        Ok(())
    }

    fn parse_create_market_log(&mut self, market_id: FixedBytes<32>, data: &[u8]) {
        if data.len() < 160 {
            return;
        }

        let loan_token = Address::from_slice(&data[12..32]);
        let collateral_token = Address::from_slice(&data[44..64]);
        let oracle = Address::from_slice(&data[76..96]);
        let irm = Address::from_slice(&data[108..128]);
        let lltv = U256::from_be_slice(&data[128..160]);

        let (loan_sym, loan_dec) = self.config.tokens.get(&loan_token)
            .cloned()
            .unwrap_or_else(|| (format!("{}", &format!("{loan_token}")[..8]), 18));
        let (col_sym, col_dec) = self.config.tokens.get(&collateral_token)
            .cloned()
            .unwrap_or_else(|| (format!("{}", &format!("{collateral_token}")[..8]), 18));

        self.markets.insert(market_id, MorphoMarket {
            id: market_id,
            loan_token,
            collateral_token,
            oracle,
            irm,
            lltv,
            total_borrow_assets: 0,
            total_borrow_shares: 0,
            total_supply_assets: 0,
            oracle_price: U256::ZERO,
            loan_symbol: loan_sym,
            collateral_symbol: col_sym,
            loan_decimals: loan_dec,
            collateral_decimals: col_dec,
        });

        self.borrowers_by_market.entry(market_id).or_default();
    }

    /// Discover borrowers via Borrow events
    async fn discover_borrowers<P: Provider + Clone>(&mut self, provider: &P) -> Result<()> {
        let borrow_topic = alloy::primitives::keccak256(
            b"Borrow(bytes32,address,address,address,uint256,uint256)",
        );

        let current_block = provider.get_block_number().await?;
        let start = if self.last_scanned_block > 0 {
            self.last_scanned_block + 1
        } else {
            self.config.start_block
        };

        if start >= current_block {
            return Ok(());
        }

        let chunk = self.config.get_logs_chunk_size;
        let prev_count = self.total_borrowers();

        info!(
            chain = %self.config.chain_name,
            from = start, to = current_block,
            "Scanning Morpho Borrow events"
        );

        let mut from = start;
        while from <= current_block {
            let to = (from + chunk - 1).min(current_block);

            let filter = Filter::new()
                .address(self.config.morpho_address)
                .event_signature(borrow_topic)
                .from_block(from)
                .to_block(to);

            match provider.get_logs(&filter).await {
                Ok(logs) => {
                    for log in &logs {
                        let topics = log.topics();
                        if topics.len() >= 3 {
                            let market_id = topics[1];
                            let borrower = Address::from_slice(&topics[2][12..]);
                            self.borrowers_by_market.entry(market_id).or_default().insert(borrower);
                        }
                    }
                }
                Err(e) => {
                    warn!(chain = %self.config.chain_name, from, to, "Borrow getLogs error: {e}");
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            from = to + 1;
        }

        self.last_scanned_block = current_block;
        let new_count = self.total_borrowers() - prev_count;

        info!(
            chain = %self.config.chain_name,
            total = self.total_borrowers(),
            new = new_count,
            "Morpho borrower discovery complete"
        );

        self.save_cache();
        Ok(())
    }

    /// Full scan: refresh market states, oracle prices, and all positions
    pub async fn full_scan<P: Provider + Clone>(
        &mut self,
        provider: &P,
    ) -> Result<Vec<MorphoCandidate>> {
        let start = std::time::Instant::now();
        self.scan_count += 1;

        if self.markets.is_empty() {
            return Ok(vec![]);
        }

        // 1. Refresh market states via Multicall
        self.refresh_market_states(provider).await?;

        // 2. Refresh oracle prices via Multicall
        self.refresh_oracle_prices(provider).await?;

        // 3. Scan all positions
        let candidates = self.scan_positions(provider).await?;

        self.last_scan_ms = start.elapsed().as_millis() as u64;
        self.total_candidates_found += candidates.len() as u64;

        info!(
            chain = %self.config.chain_name,
            scan = self.scan_count,
            ms = self.last_scan_ms,
            markets = self.markets.len(),
            borrowers = self.total_borrowers(),
            at_risk = self.at_risk_set.len(),
            liquidatable = candidates.len(),
            "Morpho scan complete"
        );

        Ok(candidates)
    }

    /// Quick scan: only re-check at-risk positions
    pub async fn quick_scan<P: Provider + Clone>(
        &mut self,
        provider: &P,
    ) -> Result<Vec<MorphoCandidate>> {
        if self.at_risk_set.is_empty() {
            return Ok(vec![]);
        }

        self.refresh_oracle_prices(provider).await?;
        self.refresh_market_states(provider).await?;

        let at_risk: Vec<(FixedBytes<32>, Address)> = self.at_risk_set.iter().cloned().collect();
        let mut candidates = Vec::new();
        let mut new_at_risk = HashSet::new();

        for (market_id, borrower) in &at_risk {
            if let Some(pos) = self.evaluate_position(provider, *market_id, *borrower).await? {
                if pos.health_factor < 1.2 {
                    new_at_risk.insert((*market_id, *borrower));
                }
                if pos.health_factor < 1.0 && pos.health_factor > 0.0 {
                    if let Some(c) = self.build_candidate(*market_id, &pos) {
                        candidates.push(c);
                    }
                }
                self.positions.insert((*market_id, *borrower), pos);
            }
        }

        self.at_risk_set = new_at_risk;
        Ok(candidates)
    }

    /// Incremental borrower discovery
    pub async fn incremental_discover<P: Provider + Clone>(
        &mut self,
        provider: &P,
        current_block: u64,
    ) -> Result<u32> {
        let safe_block = current_block.saturating_sub(2);
        let start = self.last_scanned_block + 1;
        if start >= safe_block {
            return Ok(0);
        }

        let end = safe_block.min(start + self.config.get_logs_chunk_size - 1);

        let borrow_topic = alloy::primitives::keccak256(
            b"Borrow(bytes32,address,address,address,uint256,uint256)",
        );

        let filter = Filter::new()
            .address(self.config.morpho_address)
            .event_signature(borrow_topic)
            .from_block(start)
            .to_block(end);

        let mut new_count = 0u32;
        match provider.get_logs(&filter).await {
            Ok(logs) => {
                for log in &logs {
                    let topics = log.topics();
                    if topics.len() >= 3 {
                        let market_id = topics[1];
                        let borrower = Address::from_slice(&topics[2][12..]);
                        if self.borrowers_by_market.entry(market_id).or_default().insert(borrower) {
                            new_count += 1;
                        }
                    }
                }
            }
            Err(e) => {
                debug!(chain = %self.config.chain_name, "Morpho incremental discover error: {e}");
            }
        }

        self.last_scanned_block = end;
        if new_count > 0 {
            info!(chain = %self.config.chain_name, new = new_count, "New Morpho borrowers");
            self.save_cache();
        }
        Ok(new_count)
    }

    // ── Internal ──

    async fn refresh_market_states<P: Provider + Clone>(&mut self, provider: &P) -> Result<()> {
        let market_ids: Vec<FixedBytes<32>> = self.markets.keys().cloned().collect();
        if market_ids.is_empty() {
            return Ok(());
        }

        let requests: Vec<MulticallRequest> = market_ids.iter().map(|id| {
            let call_data = IMorpho::marketCall { id: *id }.abi_encode();
            MulticallRequest {
                target: self.config.morpho_address,
                call_data: Bytes::from(call_data),
            }
        }).collect();

        let results = multicall_aggregate(provider, self.config.multicall3, &requests, 200).await?;

        for (i, result) in results.iter().enumerate() {
            if !result.success || result.return_data.is_empty() {
                continue;
            }
            if let Ok(decoded) = IMorpho::marketCall::abi_decode_returns(&result.return_data) {
                if let Some(market) = self.markets.get_mut(&market_ids[i]) {
                    market.total_supply_assets = decoded.totalSupplyAssets;
                    market.total_borrow_assets = decoded.totalBorrowAssets;
                    market.total_borrow_shares = decoded.totalBorrowShares;
                }
            }
        }

        Ok(())
    }

    async fn refresh_oracle_prices<P: Provider + Clone>(&mut self, provider: &P) -> Result<()> {
        let market_ids: Vec<FixedBytes<32>> = self.markets.keys().cloned().collect();
        let oracle_addrs: Vec<Address> = market_ids.iter()
            .filter_map(|id| self.markets.get(id).map(|m| m.oracle))
            .collect();

        if oracle_addrs.is_empty() {
            return Ok(());
        }

        let requests: Vec<MulticallRequest> = oracle_addrs.iter().map(|addr| {
            let call_data = IMorphoOracle::priceCall {}.abi_encode();
            MulticallRequest {
                target: *addr,
                call_data: Bytes::from(call_data),
            }
        }).collect();

        let results = multicall_aggregate(provider, self.config.multicall3, &requests, 200).await?;

        for (i, result) in results.iter().enumerate() {
            if !result.success || result.return_data.is_empty() {
                continue;
            }
            if let Ok(decoded) = IMorphoOracle::priceCall::abi_decode_returns(&result.return_data) {
                if let Some(market) = self.markets.get_mut(&market_ids[i]) {
                    market.oracle_price = decoded;
                }
            }
        }

        Ok(())
    }

    async fn scan_positions<P: Provider + Clone>(
        &mut self,
        provider: &P,
    ) -> Result<Vec<MorphoCandidate>> {
        let mut all_requests = Vec::new();
        let mut request_index: Vec<(FixedBytes<32>, Address)> = Vec::new();

        for (market_id, borrowers) in &self.borrowers_by_market {
            for borrower in borrowers {
                let call_data = IMorpho::positionCall {
                    id: *market_id,
                    user: *borrower,
                }.abi_encode();
                all_requests.push(MulticallRequest {
                    target: self.config.morpho_address,
                    call_data: Bytes::from(call_data),
                });
                request_index.push((*market_id, *borrower));
            }
        }

        if all_requests.is_empty() {
            return Ok(vec![]);
        }

        let results = multicall_aggregate(provider, self.config.multicall3, &all_requests, 500).await?;

        let mut candidates = Vec::new();
        let mut new_at_risk = HashSet::new();

        for (i, result) in results.iter().enumerate() {
            if !result.success || result.return_data.is_empty() {
                continue;
            }

            let (market_id, borrower) = request_index[i];

            let decoded = match IMorpho::positionCall::abi_decode_returns(&result.return_data) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let borrow_shares = decoded.borrowShares;
            let collateral = decoded.collateral;

            // Skip positions with no borrows
            if borrow_shares == 0 {
                self.positions.remove(&(market_id, borrower));
                continue;
            }

            let Some(market) = self.markets.get(&market_id) else { continue };

            // Calculate borrowed assets from shares
            let borrowed_assets = if market.total_borrow_shares > 0 {
                (borrow_shares as u128) * market.total_borrow_assets / market.total_borrow_shares
            } else {
                0
            };

            // Calculate health factor
            let hf = self.calculate_hf(market, collateral, borrowed_assets);

            let loan_dec = market.loan_decimals;
            let borrowed_f64 = borrowed_assets as f64 / 10f64.powi(loan_dec as i32);

            let pos = MorphoPosition {
                market_id,
                borrower,
                borrow_shares,
                collateral,
                borrowed_assets: borrowed_f64,
                collateral_value_usd: 0.0, // TODO: compute from oracle
                health_factor: hf,
            };

            if hf < 1.2 && hf > 0.0 {
                new_at_risk.insert((market_id, borrower));
            }

            if hf < 1.0 && hf > 0.0 {
                if let Some(c) = self.build_candidate(market_id, &pos) {
                    info!(
                        chain = %self.config.chain_name,
                        market = %&format!("{}", market_id)[..10],
                        borrower = %&format!("{}", borrower)[..10],
                        hf = format!("{:.4}", hf),
                        borrowed = format!("{:.2}", borrowed_f64),
                        pair = format!("{}/{}", market.collateral_symbol, market.loan_symbol),
                        "MORPHO CANDIDATE"
                    );
                    candidates.push(c);
                }
            }

            self.positions.insert((market_id, borrower), pos);
        }

        self.at_risk_set = new_at_risk;
        Ok(candidates)
    }

    async fn evaluate_position<P: Provider + Clone>(
        &self,
        provider: &P,
        market_id: FixedBytes<32>,
        borrower: Address,
    ) -> Result<Option<MorphoPosition>> {
        let morpho = IMorpho::new(self.config.morpho_address, provider.clone());
        let pos_result = morpho.position(market_id, borrower).call().await?;

        if pos_result.borrowShares == 0 {
            return Ok(None);
        }

        let Some(market) = self.markets.get(&market_id) else {
            return Ok(None);
        };

        let borrowed_assets = if market.total_borrow_shares > 0 {
            (pos_result.borrowShares as u128) * market.total_borrow_assets / market.total_borrow_shares
        } else {
            0
        };

        let hf = self.calculate_hf(market, pos_result.collateral, borrowed_assets);
        let borrowed_f64 = borrowed_assets as f64 / 10f64.powi(market.loan_decimals as i32);

        Ok(Some(MorphoPosition {
            market_id,
            borrower,
            borrow_shares: pos_result.borrowShares,
            collateral: pos_result.collateral,
            borrowed_assets: borrowed_f64,
            collateral_value_usd: 0.0,
            health_factor: hf,
        }))
    }

    fn calculate_hf(&self, market: &MorphoMarket, collateral: u128, borrowed_assets: u128) -> f64 {
        if borrowed_assets == 0 || market.oracle_price.is_zero() {
            return f64::MAX;
        }

        // HF = collateral * oracle_price * LLTV / (borrowed_assets * ORACLE_PRICE_SCALE * WAD)
        // Break into steps to avoid overflow:
        // collateral_value = collateral * oracle_price / ORACLE_PRICE_SCALE (in loan token units)
        // max_borrow = collateral_value * LLTV / WAD
        // hf = max_borrow / borrowed_assets

        let collateral_u256 = U256::from(collateral);
        let borrowed_u256 = U256::from(borrowed_assets);
        let oracle_scale = U256::from(ORACLE_PRICE_SCALE);
        let wad = U256::from(WAD);

        // collateral_value in loan token base units
        let collateral_value = collateral_u256 * market.oracle_price / oracle_scale;

        // max_borrow = collateral_value * lltv / WAD
        let max_borrow = collateral_value * market.lltv / wad;

        // HF = max_borrow / borrowed_assets (as f64)
        if borrowed_u256.is_zero() {
            return f64::MAX;
        }

        // Use WAD precision: hf_wad = max_borrow * WAD / borrowed_assets
        let hf_wad = max_borrow * wad / borrowed_u256;
        u256_to_f64(hf_wad) / WAD as f64
    }

    fn build_candidate(&self, market_id: FixedBytes<32>, pos: &MorphoPosition) -> Option<MorphoCandidate> {
        let market = self.markets.get(&market_id)?;

        // Estimate profit: collateral value * LIF - borrowed
        // LIF = min(0.15, 1/LLTV - 1) in WAD
        let lltv_f = u256_to_f64(market.lltv) / WAD as f64;
        let lif = (1.0 / lltv_f - 1.0).min(0.15);

        let profit_est = pos.borrowed_assets * lif;

        // Compute max seizable collateral from debt (NOT all collateral).
        // Morpho reverts if seized_assets converts to more repaidShares than borrower has.
        // Formula: seized = borrowed_assets * LIF_factor * ORACLE_PRICE_SCALE / oracle_price
        // LIF_factor = min(1.15e18, WAD^2 / (WAD - 0.3e18 * (WAD - LLTV) / WAD))
        let seized_assets = self.compute_max_seized(market, pos);

        Some(MorphoCandidate {
            market_id,
            borrower: pos.borrower,
            health_factor: pos.health_factor,
            borrowed_assets: pos.borrowed_assets,
            collateral_amount: pos.collateral,
            seized_assets,
            estimated_profit_usd: profit_est,
            loan_token: market.loan_token,
            collateral_token: market.collateral_token,
            loan_symbol: market.loan_symbol.clone(),
            collateral_symbol: market.collateral_symbol.clone(),
            oracle: market.oracle,
            irm: market.irm,
            lltv: market.lltv,
        })
    }

    /// Compute max collateral to seize based on borrower's debt.
    /// Morpho's liquidate(seizedAssets, 0) computes repaidShares from seized amount.
    /// If repaidShares > borrower.borrowShares → arithmetic underflow → revert.
    /// So we must compute: seized ≤ debt_value * LIF * ORACLE_PRICE_SCALE / oracle_price
    fn compute_max_seized(&self, market: &MorphoMarket, pos: &MorphoPosition) -> U256 {
        if market.oracle_price.is_zero() || market.total_borrow_shares == 0 {
            return U256::from(pos.collateral);
        }

        // Precise borrowed assets from shares
        let borrowed_assets = (pos.borrow_shares as u128)
            .checked_mul(market.total_borrow_assets)
            .and_then(|v| v.checked_div(market.total_borrow_shares))
            .unwrap_or(0);
        if borrowed_assets == 0 {
            return U256::from(pos.collateral);
        }

        let wad = U256::from(WAD);
        let oracle_scale = U256::from(ORACLE_PRICE_SCALE);

        // LIF = min(1.15e18, WAD / (1 - 0.3 * (1 - LLTV/WAD)))
        // = min(1.15e18, WAD^2 / (WAD - CURSOR * (WAD - LLTV) / WAD))
        let cursor = U256::from(300_000_000_000_000_000u128); // 0.3e18
        let max_lif = U256::from(1_150_000_000_000_000_000u128); // 1.15e18
        let wad_minus_lltv = wad.saturating_sub(market.lltv);
        let denom = wad.saturating_sub(cursor * wad_minus_lltv / wad);
        let lif = if denom.is_zero() {
            max_lif
        } else {
            (wad * wad / denom).min(max_lif)
        };

        // max_seized_collateral = borrowed * lif / WAD * ORACLE_PRICE_SCALE / oracle_price
        let borrowed = U256::from(borrowed_assets);
        let max_seized = borrowed * lif / wad * oracle_scale / market.oracle_price;

        // 95% safety margin for rounding (Morpho uses wDivUp/toSharesUp)
        let safe_seized = max_seized * U256::from(95u64) / U256::from(100u64);

        // Cap to actual collateral
        safe_seized.min(U256::from(pos.collateral))
    }

    // ── Cache ──

    fn cache_path(&self) -> PathBuf {
        self.data_dir.join(format!("{}-morpho-borrowers.json", self.config.chain_name.to_lowercase()))
    }

    fn load_cache(&mut self) {
        let path = self.cache_path();
        let Ok(data) = fs::read_to_string(&path) else { return };

        #[derive(serde::Deserialize)]
        struct Cache {
            #[serde(default)]
            borrowers: HashMap<String, Vec<String>>,
            #[serde(default)]
            last_block: u64,
        }

        let Ok(cache) = serde_json::from_str::<Cache>(&data) else { return };

        for (market_id_str, borrower_list) in &cache.borrowers {
            if let Ok(market_id) = market_id_str.parse::<FixedBytes<32>>() {
                let set = self.borrowers_by_market.entry(market_id).or_default();
                for addr_str in borrower_list {
                    if let Ok(addr) = addr_str.parse::<Address>() {
                        set.insert(addr);
                    }
                }
            }
        }

        if cache.last_block > 0 {
            self.last_scanned_block = cache.last_block;
        }

        info!(
            chain = %self.config.chain_name,
            borrowers = self.total_borrowers(),
            last_block = self.last_scanned_block,
            "Loaded Morpho cache"
        );
    }

    fn save_cache(&self) {
        let path = self.cache_path();
        let mut borrowers: HashMap<String, Vec<String>> = HashMap::new();
        for (market_id, set) in &self.borrowers_by_market {
            borrowers.insert(
                format!("{market_id}"),
                set.iter().map(|a| format!("{a}")).collect(),
            );
        }

        let cache = serde_json::json!({
            "chain": self.config.chain_name,
            "last_block": self.last_scanned_block,
            "borrowers": borrowers,
        });

        let tmp = path.with_extension("json.tmp");
        if let Ok(()) = fs::write(&tmp, serde_json::to_string(&cache).unwrap_or_default()) {
            fs::rename(&tmp, &path).ok();
        }
    }

    // ── Getters ──

    fn total_borrowers(&self) -> usize {
        self.borrowers_by_market.values().map(|s| s.len()).sum()
    }

    pub fn borrower_count(&self) -> usize { self.total_borrowers() }
    #[allow(dead_code)]
    pub fn market_count(&self) -> usize { self.markets.len() }
    pub fn at_risk_count(&self) -> usize { self.at_risk_set.len() }
    pub fn position_count(&self) -> usize { self.positions.len() }
    pub fn total_scans(&self) -> u64 { self.scan_count }
    pub fn last_scan_duration_ms(&self) -> u64 { self.last_scan_ms }

    pub fn at_risk_snapshots(&self) -> Vec<(Address, f64, f64, String)> {
        let mut result = Vec::new();
        for (key, pos) in &self.positions {
            if self.at_risk_set.contains(key) {
                let col_sym = self.markets.get(&key.0)
                    .map(|m| m.collateral_symbol.clone())
                    .unwrap_or_else(|| "??".to_string());
                result.push((pos.borrower, pos.health_factor, pos.borrowed_assets, col_sym));
            }
        }
        result.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }
}

fn u256_to_f64(v: U256) -> f64 {
    v.to_string().parse::<f64>().unwrap_or(0.0)
}
