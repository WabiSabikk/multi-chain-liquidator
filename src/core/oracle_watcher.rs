#![allow(dead_code)]

use alloy::primitives::{Address, FixedBytes};
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use eyre::Result;
use std::collections::HashMap;
use tracing::{debug, info};

/// Chainlink AnswerUpdated(int256 indexed current, uint256 indexed roundId, uint256 updatedAt)
const ANSWER_UPDATED_TOPIC: &str =
    "0x0559884fd3a460db3073b7fc896cc77986f16e378210dar0b10d842085acfbf35";

/// Watches for oracle price update events on-chain.
/// When a price feed updates, we know positions using that asset may become liquidatable.
pub struct OracleWatcher {
    /// Map from oracle feed address -> underlying asset address
    feed_to_asset: HashMap<Address, Address>,
    /// Last block we checked for events
    last_checked_block: u64,
}

impl OracleWatcher {
    pub fn new() -> Self {
        Self {
            feed_to_asset: HashMap::new(),
            last_checked_block: 0,
        }
    }

    /// Register known oracle feeds. On Aave V3, the AaveOracle contract
    /// wraps Chainlink/Pyth feeds. We monitor the underlying feed contracts.
    pub fn register_feed(&mut self, feed_address: Address, asset_address: Address) {
        self.feed_to_asset.insert(feed_address, asset_address);
    }

    /// Check for price update events in the given block range.
    /// Returns list of asset addresses that had price updates.
    pub async fn check_price_updates<P: Provider + Clone>(
        &mut self,
        provider: &P,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<Address>> {
        if self.feed_to_asset.is_empty() {
            return Ok(vec![]);
        }

        let feed_addresses: Vec<Address> = self.feed_to_asset.keys().copied().collect();

        // Chainlink AnswerUpdated event signature
        let answer_updated: FixedBytes<32> = alloy::primitives::keccak256(
            b"AnswerUpdated(int256,uint256,uint256)",
        );

        let filter = Filter::new()
            .address(feed_addresses)
            .event_signature(answer_updated)
            .from_block(from_block)
            .to_block(to_block);

        let logs = provider.get_logs(&filter).await?;

        let mut updated_assets = Vec::new();
        for log in &logs {
            let feed_addr = log.address();
            if let Some(asset) = self.feed_to_asset.get(&feed_addr) {
                if !updated_assets.contains(asset) {
                    updated_assets.push(*asset);
                    debug!(
                        feed = %feed_addr,
                        asset = %asset,
                        "Oracle price updated"
                    );
                }
            }
        }

        if !updated_assets.is_empty() {
            info!(
                count = updated_assets.len(),
                block_range = %format!("{from_block}-{to_block}"),
                "Oracle price updates detected"
            );
        }

        self.last_checked_block = to_block;
        Ok(updated_assets)
    }

    pub fn last_checked_block(&self) -> u64 {
        self.last_checked_block
    }

    pub fn set_last_checked_block(&mut self, block: u64) {
        self.last_checked_block = block;
    }
}

/// Discover Chainlink feed addresses by reading the AaveOracle's getSourceOfAsset
/// (if available) or by looking at known feed addresses.
/// Returns map of feed_address -> asset_address.
pub async fn discover_oracle_feeds<P: Provider + Clone>(
    _provider: &P,
    _oracle_address: Address,
    asset_addresses: &[Address],
) -> Result<HashMap<Address, Address>> {
    // For now, we detect price changes via the AaveOracle directly
    // by comparing prices between blocks. This is simpler and works
    // across all oracle implementations (Chainlink, Pyth, Chaos Labs).
    //
    // A more advanced approach would be to read the specific price feed
    // addresses from each oracle implementation and subscribe to their
    // events directly for sub-block latency.
    let _ = asset_addresses;
    Ok(HashMap::new())
}
