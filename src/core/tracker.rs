use alloy::primitives::{Address, FixedBytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::core::dashboard::now_unix;
use crate::core::types::MissedOppSnapshot;

/// Competitor liquidation record (written to JSONL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorLiq {
    pub ts: u64,
    pub block: u64,
    pub chain: String,
    pub tx: String,
    pub liquidator: String,
    pub user: String,
    pub col_asset: String,
    pub col_sym: String,
    pub debt_asset: String,
    pub debt_sym: String,
    pub debt_covered: String,
    pub col_seized: String,
    pub ours: bool,
}

/// Missed opportunity record (written to JSONL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissedOppRecord {
    pub ts: u64,
    pub chain: String,
    pub target: String,
    pub hf: f64,
    pub debt_usd: f64,
    pub pair: String,
    pub reason: String,
    pub profit_est: f64,
    pub resolved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competitor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competitor_tx: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_s: Option<u64>,
}

/// Tracker statistics (for dashboard)
#[derive(Debug, Clone, Default, Serialize)]
pub struct TrackerStats {
    pub total_competitor_liqs: u64,
    pub total_our_liqs: u64,
    pub total_missed: u64,
    pub total_missed_with_competitor: u64,
    pub total_missed_expired: u64,
    pub estimated_missed_profit_usd: f64,
}

/// Morpho market info for decoding liquidation events
pub struct MorphoMarketInfo {
    pub collateral_token: Address,
    pub loan_token: Address,
    pub collateral_symbol: String,
    pub loan_symbol: String,
}

struct PendingMissed {
    snapshot: MissedOppSnapshot,
    recorded_at: u64,
    recorded_block: u64,
}

const MAX_RECENT_LIQS: usize = 500;
const MISSED_EXPIRE_SECONDS: u64 = 120;

/// Tracks competitor liquidations and missed opportunities per chain/pool.
pub struct CompetitorTracker {
    chain_name: String,
    our_addresses: HashSet<Address>,
    last_scanned_block: u64,
    get_logs_chunk_size: u64,
    token_symbols: HashMap<Address, String>,

    recent_liqs: VecDeque<CompetitorLiq>,
    pending_missed: Vec<PendingMissed>,
    pub stats: TrackerStats,
    data_dir: PathBuf,
}

impl CompetitorTracker {
    pub fn new(
        chain_name: String,
        our_addresses: HashSet<Address>,
        token_symbols: HashMap<Address, String>,
        current_block: u64,
        get_logs_chunk_size: u64,
    ) -> Self {
        let data_dir = PathBuf::from("data");
        std::fs::create_dir_all(&data_dir).ok();
        Self {
            chain_name,
            our_addresses,
            last_scanned_block: current_block,
            get_logs_chunk_size,
            token_symbols,
            recent_liqs: VecDeque::new(),
            pending_missed: Vec::new(),
            stats: TrackerStats::default(),
            data_dir,
        }
    }

    /// Scan for Aave V2/V3 LiquidationCall events in recent blocks.
    pub async fn scan_aave_liquidations<P: Provider + Clone>(
        &mut self,
        provider: &P,
        pool_address: Address,
        current_block: u64,
    ) -> Result<u32> {
        let safe_block = current_block.saturating_sub(2);
        let start = self.last_scanned_block + 1;
        if start >= safe_block {
            return Ok(0);
        }
        let end = safe_block.min(start + self.get_logs_chunk_size - 1);

        let topic = alloy::primitives::keccak256(
            b"LiquidationCall(address,address,address,uint256,uint256,address,bool)",
        );

        let filter = Filter::new()
            .address(pool_address)
            .event_signature(topic)
            .from_block(start)
            .to_block(end);

        let logs = match provider.get_logs(&filter).await {
            Ok(logs) => logs,
            Err(e) => {
                debug!(chain = %self.chain_name, "Tracker getLogs error: {e}");
                return Ok(0);
            }
        };

        let mut count = 0u32;
        for log in &logs {
            let topics = log.topics();
            if topics.len() < 4 {
                continue;
            }
            let data = &log.data().data;
            if data.len() < 128 {
                continue;
            }

            let collateral_asset = Address::from_slice(&topics[1][12..]);
            let debt_asset = Address::from_slice(&topics[2][12..]);
            let user = Address::from_slice(&topics[3][12..]);

            let debt_covered = U256::from_be_slice(&data[0..32]);
            let col_seized = U256::from_be_slice(&data[32..64]);
            let liquidator = Address::from_slice(&data[76..96]);

            let tx_hash = log
                .transaction_hash
                .map(|h| format!("{h}"))
                .unwrap_or_default();
            let block = log.block_number.unwrap_or(0);
            let timestamp = log.block_timestamp.unwrap_or_else(now_unix);
            let is_ours = self.our_addresses.contains(&liquidator);

            let col_sym = self.get_symbol(collateral_asset);
            let debt_sym = self.get_symbol(debt_asset);

            let liq = CompetitorLiq {
                ts: timestamp,
                block,
                chain: self.chain_name.clone(),
                tx: tx_hash,
                liquidator: format!("{liquidator}"),
                user: format!("{user}"),
                col_asset: format!("{collateral_asset}"),
                col_sym: col_sym.clone(),
                debt_asset: format!("{debt_asset}"),
                debt_sym: debt_sym.clone(),
                debt_covered: format!("{debt_covered}"),
                col_seized: format!("{col_seized}"),
                ours: is_ours,
            };

            if is_ours {
                self.stats.total_our_liqs += 1;
                info!(
                    chain = %self.chain_name, block,
                    "OUR liquidation: {debt_sym}->{col_sym} user={}",
                    &format!("{user}")[..10]
                );
            } else {
                self.stats.total_competitor_liqs += 1;
                info!(
                    chain = %self.chain_name, block,
                    liquidator = %&format!("{liquidator}")[..10],
                    "COMPETITOR liquidation: {debt_sym}->{col_sym} user={}",
                    &format!("{user}")[..10]
                );
            }

            self.append_jsonl("competitor-liqs.jsonl", &liq);
            self.recent_liqs.push_front(liq);
            while self.recent_liqs.len() > MAX_RECENT_LIQS {
                self.recent_liqs.pop_back();
            }

            count += 1;
        }

        self.last_scanned_block = end;
        Ok(count)
    }

    /// Scan for Morpho Liquidate events in recent blocks.
    pub async fn scan_morpho_liquidations<P: Provider + Clone>(
        &mut self,
        provider: &P,
        morpho_address: Address,
        current_block: u64,
        markets: &HashMap<FixedBytes<32>, MorphoMarketInfo>,
    ) -> Result<u32> {
        let safe_block = current_block.saturating_sub(2);
        let start = self.last_scanned_block + 1;
        if start >= safe_block {
            return Ok(0);
        }
        let end = safe_block.min(start + self.get_logs_chunk_size - 1);

        let topic = alloy::primitives::keccak256(
            b"Liquidate(bytes32,address,address,uint256,uint256,uint256,uint256,uint256)",
        );

        let filter = Filter::new()
            .address(morpho_address)
            .event_signature(topic)
            .from_block(start)
            .to_block(end);

        let logs = match provider.get_logs(&filter).await {
            Ok(logs) => logs,
            Err(e) => {
                debug!(chain = %self.chain_name, "Morpho tracker getLogs error: {e}");
                return Ok(0);
            }
        };

        let mut count = 0u32;
        for log in &logs {
            let topics = log.topics();
            if topics.len() < 4 {
                continue;
            }
            let data = &log.data().data;
            if data.len() < 160 {
                continue;
            }

            let market_id = topics[1];
            let caller = Address::from_slice(&topics[2][12..]);
            let borrower = Address::from_slice(&topics[3][12..]);

            let repaid_assets = U256::from_be_slice(&data[0..32]);
            let seized_assets = U256::from_be_slice(&data[64..96]);

            let tx_hash = log
                .transaction_hash
                .map(|h| format!("{h}"))
                .unwrap_or_default();
            let block = log.block_number.unwrap_or(0);
            let timestamp = log.block_timestamp.unwrap_or_else(now_unix);
            let is_ours = self.our_addresses.contains(&caller);

            let (col_sym, debt_sym, col_addr, debt_addr) =
                if let Some(m) = markets.get(&market_id) {
                    (
                        m.collateral_symbol.clone(),
                        m.loan_symbol.clone(),
                        format!("{}", m.collateral_token),
                        format!("{}", m.loan_token),
                    )
                } else {
                    let mid = format!("{market_id}");
                    (
                        mid[..8].to_string(),
                        "?".to_string(),
                        String::new(),
                        String::new(),
                    )
                };

            let liq = CompetitorLiq {
                ts: timestamp,
                block,
                chain: self.chain_name.clone(),
                tx: tx_hash,
                liquidator: format!("{caller}"),
                user: format!("{borrower}"),
                col_asset: col_addr,
                col_sym: col_sym.clone(),
                debt_asset: debt_addr,
                debt_sym: debt_sym.clone(),
                debt_covered: format!("{repaid_assets}"),
                col_seized: format!("{seized_assets}"),
                ours: is_ours,
            };

            if is_ours {
                self.stats.total_our_liqs += 1;
            } else {
                self.stats.total_competitor_liqs += 1;
                info!(
                    chain = %self.chain_name, block,
                    liquidator = %&format!("{caller}")[..10],
                    "COMPETITOR Morpho liq: {col_sym}/{debt_sym}"
                );
            }

            self.append_jsonl("competitor-liqs.jsonl", &liq);
            self.recent_liqs.push_front(liq);
            while self.recent_liqs.len() > MAX_RECENT_LIQS {
                self.recent_liqs.pop_back();
            }

            count += 1;
        }

        self.last_scanned_block = end;
        Ok(count)
    }

    /// Record a missed opportunity from executor.
    /// Only counts as "missed" if profit >= $0.01 (skip dust that inflates the counter).
    pub fn record_missed(&mut self, snapshot: MissedOppSnapshot, current_block: u64) {
        if snapshot.estimated_profit < 0.01 {
            // Dust — don't count in stats or log, just silently track for analytics
            self.pending_missed.push(PendingMissed {
                snapshot,
                recorded_at: now_unix(),
                recorded_block: current_block,
            });
            return;
        }

        self.stats.total_missed += 1;
        self.stats.estimated_missed_profit_usd += snapshot.estimated_profit;

        info!(
            chain = %self.chain_name,
            target = %&format!("{}", snapshot.target)[..10],
            reason = %snapshot.reason,
            profit = format!("${:.2}", snapshot.estimated_profit),
            "MISSED OPPORTUNITY"
        );

        self.pending_missed.push(PendingMissed {
            snapshot,
            recorded_at: now_unix(),
            recorded_block: current_block,
        });
    }

    /// Resolve pending missed opps against recent competitor liquidations.
    pub fn resolve_pending(&mut self, _current_block: u64) {
        if self.pending_missed.is_empty() {
            return;
        }

        let now = now_unix();
        let mut resolved_indices = Vec::new();

        for (i, pending) in self.pending_missed.iter().enumerate() {
            let target_str = format!("{}", pending.snapshot.target);

            // Check if WE eventually liquidated this user (not a true miss)
            let our_liq = self.recent_liqs.iter().any(|liq| {
                liq.user == target_str && liq.block >= pending.recorded_block && liq.ours
            });
            if our_liq {
                resolved_indices.push(i);
                continue;
            }

            // Check if a competitor liquidated this user
            let competitor = self.recent_liqs.iter().find(|liq| {
                liq.user == target_str && liq.block >= pending.recorded_block && !liq.ours
            });

            if let Some(competitor_liq) = competitor {
                self.stats.total_missed_with_competitor += 1;
                let latency = now.saturating_sub(pending.recorded_at);

                let record = MissedOppRecord {
                    ts: pending.recorded_at,
                    chain: self.chain_name.clone(),
                    target: target_str,
                    hf: pending.snapshot.health_factor,
                    debt_usd: pending.snapshot.debt_usd,
                    pair: format!(
                        "{}->{}",
                        pending.snapshot.debt_symbol, pending.snapshot.collateral_symbol
                    ),
                    reason: pending.snapshot.reason.clone(),
                    profit_est: pending.snapshot.estimated_profit,
                    resolved: true,
                    competitor: Some(competitor_liq.liquidator.clone()),
                    competitor_tx: Some(competitor_liq.tx.clone()),
                    latency_s: Some(latency),
                };

                info!(
                    chain = %self.chain_name,
                    competitor = %&competitor_liq.liquidator[..10],
                    latency_s = latency,
                    profit = format!("${:.2}", pending.snapshot.estimated_profit),
                    "Missed opp RESOLVED: competitor beat us"
                );

                self.append_jsonl("missed-opps.jsonl", &record);
                resolved_indices.push(i);
            } else if now.saturating_sub(pending.recorded_at) > MISSED_EXPIRE_SECONDS {
                // Expired — position likely recovered
                self.stats.total_missed_expired += 1;

                let record = MissedOppRecord {
                    ts: pending.recorded_at,
                    chain: self.chain_name.clone(),
                    target: target_str,
                    hf: pending.snapshot.health_factor,
                    debt_usd: pending.snapshot.debt_usd,
                    pair: format!(
                        "{}->{}",
                        pending.snapshot.debt_symbol, pending.snapshot.collateral_symbol
                    ),
                    reason: pending.snapshot.reason.clone(),
                    profit_est: pending.snapshot.estimated_profit,
                    resolved: true,
                    competitor: None,
                    competitor_tx: None,
                    latency_s: None,
                };

                self.append_jsonl("missed-opps.jsonl", &record);
                resolved_indices.push(i);
            }
        }

        // Remove resolved (reverse order preserves indices for swap_remove)
        resolved_indices.sort_unstable();
        for i in resolved_indices.into_iter().rev() {
            self.pending_missed.swap_remove(i);
        }
    }

    pub fn stats(&self) -> &TrackerStats {
        &self.stats
    }

    fn get_symbol(&self, addr: Address) -> String {
        self.token_symbols
            .get(&addr)
            .cloned()
            .unwrap_or_else(|| {
                let s = format!("{addr}");
                s[..8].to_string()
            })
    }

    fn append_jsonl<T: Serialize>(&self, filename: &str, record: &T) {
        if let Ok(line) = serde_json::to_string(record) {
            let path = self.data_dir.join(filename);
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                let _ = writeln!(f, "{line}");
            }
        }
    }
}
