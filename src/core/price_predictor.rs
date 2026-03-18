//! CEX WebSocket Price Feed — predicts on-chain oracle prices 1-3 seconds ahead.
//!
//! HyperLend oracle reads HyperCore System Oracle, which is a weighted median of:
//! - Binance (weight=3)
//! - OKX (weight=2)
//! - Bybit (weight=2)
//! Plus ~5 other exchanges.
//!
//! By connecting to the same CEX feeds and computing the same weighted median locally,
//! we can predict the next oracle price update BEFORE it hits on-chain.
//!
//! Integration with the liquidation pipeline:
//! 1. PricePredictor runs in background, receiving CEX price ticks
//! 2. On each tick, re-computes HF for at-risk positions using predicted price
//! 3. When predicted HF < threshold, sends PredictedCandidate to executor channel
//! 4. Executor fires pre-built TX immediately (from TxCache)

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

use alloy::primitives::{Address, U256};

/// A predicted liquidation candidate from CEX price analysis
#[derive(Debug, Clone)]
pub struct PredictedCandidate {
    pub borrower: Address,
    pub predicted_hf: f64,
    pub collateral_asset: Address,
    pub debt_asset: Address,
    pub collateral_symbol: String,
    pub debt_symbol: String,
    pub debt_to_cover: U256,
    pub predicted_price: f64,
    pub oracle_price: f64,
    pub confidence: f64,
}

/// CEX price data point
#[derive(Debug, Clone)]
struct CexPrice {
    price: f64,
    timestamp_ms: u64,
    exchange: Exchange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Exchange {
    Binance,
    Okx,
    Bybit,
}

impl Exchange {
    fn weight(&self) -> u32 {
        match self {
            Exchange::Binance => 3,
            Exchange::Okx => 2,
            Exchange::Bybit => 2,
        }
    }
}

/// At-risk position cached for fast HF recomputation
#[derive(Debug, Clone)]
pub struct AtRiskEntry {
    pub borrower: Address,
    pub health_factor: f64,
    pub total_collateral_base: f64,
    pub total_debt_base: f64,
    pub collateral_asset: Address,
    pub collateral_symbol: String,
    pub collateral_price_usd: f64,
    pub debt_asset: Address,
    pub debt_symbol: String,
    pub debt_price_usd: f64,
    pub debt_to_cover: U256,
    pub liquidation_threshold_bps: u64,
    /// Which price (HYPE, ETH, BTC) affects this position
    pub price_sensitive_asset: String,
}

/// Shared state for updating at-risk positions from monitor
pub type SharedAtRisk = Arc<RwLock<Vec<AtRiskEntry>>>;

/// Price predictor: connects to CEX WS feeds, predicts oracle prices,
/// fires liquidation triggers when predicted HF drops below threshold.
pub struct PricePredictor {
    /// Latest prices from each exchange per trading pair
    prices: HashMap<(String, Exchange), CexPrice>,
    /// Channel to send predicted candidates to executor
    trigger_tx: mpsc::Sender<PredictedCandidate>,
    /// At-risk positions (updated by monitor)
    at_risk: SharedAtRisk,
    /// Current on-chain oracle prices
    oracle_prices: HashMap<String, f64>,
    /// Threshold: fire when predicted HF below this
    trigger_threshold: f64,
    /// Stats
    total_ticks: u64,
    total_triggers: u64,
    total_false_positives: u64,
}

impl PricePredictor {
    pub fn new(
        trigger_tx: mpsc::Sender<PredictedCandidate>,
        at_risk: SharedAtRisk,
    ) -> Self {
        Self {
            prices: HashMap::new(),
            trigger_tx,
            at_risk,
            oracle_prices: HashMap::new(),
            trigger_threshold: 0.998, // Fire slightly before HF=1.0
            total_ticks: 0,
            total_triggers: 0,
            total_false_positives: 0,
        }
    }

    /// Update on-chain oracle price (called by monitor after each scan)
    pub fn update_oracle_price(&mut self, asset: &str, price: f64) {
        self.oracle_prices.insert(asset.to_string(), price);
    }

    /// Process a CEX price update
    async fn on_price_update(&mut self, pair: &str, price: f64, exchange: Exchange) {
        self.total_ticks += 1;

        let key = (pair.to_string(), exchange);
        self.prices.insert(key, CexPrice {
            price,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            exchange,
        });

        // Compute weighted median
        let predicted_price = self.compute_weighted_median(pair);
        if predicted_price == 0.0 {
            return;
        }

        let oracle_price = self.oracle_prices.get(pair).copied().unwrap_or(0.0);
        if oracle_price == 0.0 {
            return;
        }

        let drift_pct = (predicted_price - oracle_price) / oracle_price * 100.0;

        // Log every 100th tick or significant drift
        if self.total_ticks % 100 == 0 || drift_pct.abs() > 0.5 {
            debug!(
                pair,
                predicted = format!("{:.4}", predicted_price),
                oracle = format!("{:.4}", oracle_price),
                drift_pct = format!("{:.4}%", drift_pct),
                ticks = self.total_ticks,
                "CEX price update"
            );
        }

        // Check at-risk positions against predicted price
        let at_risk = self.at_risk.read().await;
        for entry in at_risk.iter() {
            if entry.price_sensitive_asset != pair {
                continue;
            }

            // Recompute HF with predicted price
            let predicted_hf = self.predict_hf(entry, predicted_price);

            if predicted_hf < self.trigger_threshold {
                self.total_triggers += 1;
                info!(
                    borrower = %entry.borrower,
                    pair,
                    predicted_hf = format!("{:.6}", predicted_hf),
                    current_hf = format!("{:.6}", entry.health_factor),
                    predicted_price = format!("{:.4}", predicted_price),
                    oracle_price = format!("{:.4}", oracle_price),
                    drift_pct = format!("{:.4}%", drift_pct),
                    total_triggers = self.total_triggers,
                    "CEX PREDICTION TRIGGER — predicted HF below threshold"
                );

                let candidate = PredictedCandidate {
                    borrower: entry.borrower,
                    predicted_hf,
                    collateral_asset: entry.collateral_asset,
                    debt_asset: entry.debt_asset,
                    collateral_symbol: entry.collateral_symbol.clone(),
                    debt_symbol: entry.debt_symbol.clone(),
                    debt_to_cover: entry.debt_to_cover,
                    predicted_price,
                    oracle_price,
                    confidence: 1.0 - (drift_pct.abs() / 10.0).min(1.0),
                };

                if let Err(e) = self.trigger_tx.try_send(candidate) {
                    warn!("Failed to send prediction trigger: {e}");
                }
            }
        }
    }

    /// Compute weighted median from CEX prices (same algorithm as HyperCore validators)
    fn compute_weighted_median(&self, pair: &str) -> f64 {
        let mut weighted_prices: Vec<f64> = Vec::new();

        for exchange in [Exchange::Binance, Exchange::Okx, Exchange::Bybit] {
            let key = (pair.to_string(), exchange);
            if let Some(p) = self.prices.get(&key) {
                // Skip stale prices (> 5 seconds old)
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                if now - p.timestamp_ms > 5000 {
                    continue;
                }
                // Add price N times according to weight
                for _ in 0..exchange.weight() {
                    weighted_prices.push(p.price);
                }
            }
        }

        if weighted_prices.is_empty() {
            return 0.0;
        }

        weighted_prices.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = weighted_prices.len() / 2;
        if weighted_prices.len() % 2 == 0 {
            (weighted_prices[mid - 1] + weighted_prices[mid]) / 2.0
        } else {
            weighted_prices[mid]
        }
    }

    /// Predict HF given a new price for the position's sensitive asset.
    /// Uses simplified calculation: HF ≈ (collateral_value × lt) / debt_value
    fn predict_hf(&self, entry: &AtRiskEntry, new_price: f64) -> f64 {
        if entry.total_debt_base == 0.0 || entry.collateral_price_usd == 0.0 {
            return 999.0;
        }

        // Determine how the price change affects collateral vs debt
        let lt = entry.liquidation_threshold_bps as f64 / 10000.0;

        // If collateral is the price-sensitive asset: new_price affects collateral value
        // If debt is the price-sensitive asset: new_price affects debt value
        let price_ratio = new_price / entry.collateral_price_usd;

        if entry.collateral_symbol == entry.price_sensitive_asset
            || entry.collateral_symbol.contains("HYPE")
        {
            // Collateral price changed
            let new_collateral = entry.total_collateral_base * price_ratio;
            new_collateral * lt / entry.total_debt_base
        } else if entry.debt_symbol == entry.price_sensitive_asset {
            // Debt price changed
            let debt_ratio = new_price / entry.debt_price_usd;
            let new_debt = entry.total_debt_base * debt_ratio;
            entry.total_collateral_base * lt / new_debt
        } else {
            // This position isn't directly affected
            entry.health_factor
        }
    }
}

/// Parse Binance WS trade message: {"e":"trade","s":"HYPEUSDT","p":"25.43",...}
fn parse_binance_trade(msg: &str) -> Option<(String, f64)> {
    let v: serde_json::Value = serde_json::from_str(msg).ok()?;
    let symbol = v.get("s")?.as_str()?;
    let price: f64 = v.get("p")?.as_str()?.parse().ok()?;

    // Convert HYPEUSDT -> HYPE
    let pair = symbol.strip_suffix("USDT")?;
    Some((pair.to_string(), price))
}

/// Parse OKX WS trade message: {"data":[{"px":"25.43","instId":"HYPE-USDT",...}]}
fn parse_okx_trade(msg: &str) -> Option<(String, f64)> {
    let v: serde_json::Value = serde_json::from_str(msg).ok()?;
    let data = v.get("data")?.as_array()?.first()?;
    let price: f64 = data.get("px")?.as_str()?.parse().ok()?;
    let inst_id = data.get("instId")?.as_str()?;

    // Convert HYPE-USDT -> HYPE
    let pair = inst_id.split('-').next()?;
    Some((pair.to_string(), price))
}

/// Parse Bybit WS trade message: {"data":{"s":"HYPEUSDT","p":"25.43",...}}
fn parse_bybit_trade(msg: &str) -> Option<(String, f64)> {
    let v: serde_json::Value = serde_json::from_str(msg).ok()?;
    let data = v.get("data")?;

    // Bybit can return array or object
    let trade = if data.is_array() {
        data.as_array()?.first()?
    } else {
        data
    };

    let price: f64 = trade.get("p")?.as_str()?.parse().ok()?;
    let symbol = trade.get("s")?.as_str()?;

    let pair = symbol.strip_suffix("USDT")?;
    Some((pair.to_string(), price))
}

/// Run the CEX WebSocket price feed in the background.
/// Connects to Binance, OKX, Bybit and feeds prices to PricePredictor.
pub async fn run_cex_feeds(
    predictor: Arc<RwLock<PricePredictor>>,
    assets: Vec<String>, // ["HYPE", "BTC", "ETH"]
) {
    info!(assets = ?assets, "Starting CEX WebSocket feeds");

    let predictor_binance = predictor.clone();
    let predictor_okx = predictor.clone();
    let predictor_bybit = predictor.clone();
    let assets_b = assets.clone();
    let assets_o = assets.clone();
    let assets_y = assets.clone();

    // Spawn feeds in parallel
    tokio::join!(
        run_binance_feed(predictor_binance, assets_b),
        run_okx_feed(predictor_okx, assets_o),
        run_bybit_feed(predictor_bybit, assets_y),
    );
}

/// Binance WebSocket trade stream
async fn run_binance_feed(predictor: Arc<RwLock<PricePredictor>>, assets: Vec<String>) {
    loop {
        let streams: Vec<String> = assets.iter()
            .map(|a| format!("{}usdt@trade", a.to_lowercase()))
            .collect();
        let url = format!("wss://stream.binance.com:9443/stream?streams={}", streams.join("/"));

        info!(url = %url, "Connecting to Binance WS...");

        match tokio_tungstenite::connect_async(&url).await {
            Ok((ws_stream, _)) => {
                use futures_util::StreamExt;
                let (_, mut read) = ws_stream.split();

                info!("Binance WS connected");

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                            // Binance combined stream wraps in {"stream":"...","data":{...}}
                            if let Some(data) = serde_json::from_str::<serde_json::Value>(&text)
                                .ok()
                                .and_then(|v| v.get("data").cloned())
                            {
                                if let Some((pair, price)) = parse_binance_trade(&data.to_string()) {
                                    let mut pred = predictor.write().await;
                                    pred.on_price_update(&pair, price, Exchange::Binance).await;
                                }
                            }
                        }
                        Ok(tokio_tungstenite::tungstenite::Message::Ping(data)) => {
                            debug!("Binance ping received");
                            let _ = data; // Auto-pong handled by tungstenite
                        }
                        Err(e) => {
                            warn!("Binance WS error: {e}");
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                error!("Binance WS connect failed: {e}");
            }
        }

        warn!("Binance WS disconnected, reconnecting in 5s...");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

/// OKX WebSocket trade stream
async fn run_okx_feed(predictor: Arc<RwLock<PricePredictor>>, assets: Vec<String>) {
    loop {
        let url = "wss://ws.okx.com:8443/ws/v5/public";

        info!("Connecting to OKX WS...");

        match tokio_tungstenite::connect_async(url).await {
            Ok((ws_stream, _)) => {
                use futures_util::{SinkExt, StreamExt};
                let (mut write, mut read) = ws_stream.split();

                // Subscribe to trades
                let subscribe = serde_json::json!({
                    "op": "subscribe",
                    "args": assets.iter().map(|a| {
                        serde_json::json!({
                            "channel": "trades",
                            "instId": format!("{}-USDT", a.to_uppercase())
                        })
                    }).collect::<Vec<_>>()
                });

                if let Err(e) = write.send(tokio_tungstenite::tungstenite::Message::Text(subscribe.to_string())).await {
                    error!("OKX subscribe failed: {e}");
                    continue;
                }

                info!("OKX WS connected + subscribed");

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                            if let Some((pair, price)) = parse_okx_trade(&text) {
                                let mut pred = predictor.write().await;
                                pred.on_price_update(&pair, price, Exchange::Okx).await;
                            }
                        }
                        Err(e) => {
                            warn!("OKX WS error: {e}");
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                error!("OKX WS connect failed: {e}");
            }
        }

        warn!("OKX WS disconnected, reconnecting in 5s...");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

/// Bybit WebSocket trade stream
async fn run_bybit_feed(predictor: Arc<RwLock<PricePredictor>>, assets: Vec<String>) {
    loop {
        let url = "wss://stream.bybit.com/v5/public/spot";

        info!("Connecting to Bybit WS...");

        match tokio_tungstenite::connect_async(url).await {
            Ok((ws_stream, _)) => {
                use futures_util::{SinkExt, StreamExt};
                let (mut write, mut read) = ws_stream.split();

                // Subscribe to trades
                let subscribe = serde_json::json!({
                    "op": "subscribe",
                    "args": assets.iter().map(|a| format!("publicTrade.{}USDT", a.to_uppercase())).collect::<Vec<_>>()
                });

                if let Err(e) = write.send(tokio_tungstenite::tungstenite::Message::Text(subscribe.to_string())).await {
                    error!("Bybit subscribe failed: {e}");
                    continue;
                }

                info!("Bybit WS connected + subscribed");

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                            if let Some((pair, price)) = parse_bybit_trade(&text) {
                                let mut pred = predictor.write().await;
                                pred.on_price_update(&pair, price, Exchange::Bybit).await;
                            }
                        }
                        Err(e) => {
                            warn!("Bybit WS error: {e}");
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                error!("Bybit WS connect failed: {e}");
            }
        }

        warn!("Bybit WS disconnected, reconnecting in 5s...");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
