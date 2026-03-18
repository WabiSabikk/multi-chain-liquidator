mod chains;
mod core;
mod dex;
mod protocols;

use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use eyre::Result;
use futures_util::StreamExt;
use tokio::signal;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

use crate::core::config::{ChainConfig, GlobalConfig};
use crate::core::dashboard::{
    new_shared_dashboard, now_unix, run_dashboard_server, AtRiskSnapshot, DashboardEvent,
    MonitorSnapshot, SharedDashboard,
};
use crate::protocols::aave_v3::executor::AaveV3Executor;
use crate::protocols::aave_v3::monitor::AaveV3Monitor;
use crate::protocols::morpho::executor::MorphoExecutor;
use crate::protocols::morpho::monitor::{MorphoConfig, MorphoMonitor};

fn u256_to_f64(v: alloy::primitives::U256) -> f64 {
    v.to_string().parse::<f64>().unwrap_or(0.0)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let global = GlobalConfig::from_env()?;

    info!("============================================================");
    info!("  Multi-Chain Liquidator v0.4.0 (newHeads + parallel)");
    info!("============================================================");
    info!("  Mode: {}", if global.dry_run { "DRY RUN" } else { "LIVE" });
    info!("============================================================");

    let chains: Vec<ChainConfig> = vec![
        chains::mantle::chain_config(),
        chains::ink::chain_config(),
        chains::hyperevm::chain_config(),
    ];

    let signer: PrivateKeySigner = global.private_key.parse()?;
    let signer_address = signer.address();
    info!("Wallet: {signer_address}");

    // Dashboard shared state
    let dashboard = new_shared_dashboard();
    {
        let mut state = dashboard.write().await;
        state.dry_run = global.dry_run;
    }

    {
        let mut state = dashboard.write().await;
        state.wallet_address = format!("{signer_address}");
    }

    // Spawn dashboard HTTP server
    if let Some(ref token) = global.dashboard_token {
        let dash = dashboard.clone();
        let port = global.dashboard_port;
        let token = token.clone();
        tokio::spawn(async move {
            run_dashboard_server(dash, port, token).await;
        });
        info!(port = global.dashboard_port, "Dashboard server spawned");
    } else {
        warn!("DASHBOARD_TOKEN not set — dashboard disabled");
    }

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let mut handles = Vec::new();

    for chain in &chains {
        if !chain.enabled || chain.rpc_url.is_empty() {
            warn!(chain = %chain.name, "Skipped (disabled or no RPC URL)");
            continue;
        }

        // Spawn Aave V3 tasks
        for pool_cfg in &chain.aave_pools {
            if pool_cfg.data_provider.is_zero() {
                info!(
                    chain = %chain.name,
                    pool = %pool_cfg.label,
                    "Skipped (data_provider not configured yet)"
                );
                continue;
            }

            let mut pool_chain = chain.clone();
            pool_chain.name = format!("{}/{}", chain.name, pool_cfg.label);
            pool_chain.aave_pool = pool_cfg.pool;
            pool_chain.aave_data_provider = pool_cfg.data_provider;
            pool_chain.aave_oracle = pool_cfg.oracle;
            pool_chain.liquidator_contract = pool_cfg.liquidator_contract;
            pool_chain.borrow_start_block = pool_cfg.borrow_start_block;
            pool_chain.is_v2 = pool_cfg.is_v2;
            pool_chain.base_currency_decimals = pool_cfg.base_currency_decimals;
            pool_chain.cross_token_map = pool_cfg.cross_token_map.clone();

            let dry_run = global.dry_run;
            let tg_token = global.telegram_bot_token.clone();
            let tg_chat = global.telegram_chat_id.clone();
            let pk = global.private_key.clone();
            let rx = shutdown_rx.clone();
            let dash = dashboard.clone();

            let handle = tokio::spawn(async move {
                if let Err(e) = run_aave_chain(pool_chain, pk, dry_run, tg_token, tg_chat, dash, rx).await {
                    error!("Aave chain task failed: {e}");
                }
            });

            handles.push(handle);
        }

        // Spawn Morpho task (if configured)
        if let Some(ref morpho_deploy) = chain.morpho {
            let morpho_cfg = MorphoConfig {
                morpho_address: morpho_deploy.morpho_address,
                multicall3: chain.multicall3,
                chain_name: format!("{}/Morpho", chain.name),
                start_block: morpho_deploy.start_block,
                get_logs_chunk_size: chain.get_logs_chunk_size,
                scan_interval_ms: chain.scan_interval_ms,
                poll_interval_ms: chain.poll_interval_ms,
                min_debt_usd: chain.min_debt_usd,
                min_profit_usd: chain.min_profit_usd,
                tokens: chain.tokens.iter().map(|(addr, t)| (*addr, (t.symbol.clone(), t.decimals))).collect(),
            };

            let rpc_url = chain.rpc_url.clone();
            let morpho_wss = chain.wss_url.clone();
            let pk = global.private_key.clone();
            let dry_run = global.dry_run;
            let liq_contract = morpho_deploy.liquidator_contract;
            let min_profit = chain.min_profit_usd;
            let rx = shutdown_rx.clone();
            let dash = dashboard.clone();

            let handle = tokio::spawn(async move {
                if let Err(e) = run_morpho_chain(morpho_cfg, rpc_url, morpho_wss, pk, dry_run, liq_contract, min_profit, dash, rx).await {
                    error!("Morpho chain task failed: {e}");
                }
            });

            handles.push(handle);
        }
    }

    if handles.is_empty() {
        warn!("No chains enabled. Set MANTLE_RPC_URL, INK_RPC_URL, or HYPEREVM_RPC_URL.");
        return Ok(());
    }

    info!("All chains started. Press Ctrl+C to stop.");
    signal::ctrl_c().await?;
    info!("Shutting down...");
    shutdown_tx.send(true)?;

    for handle in handles {
        handle.await.ok();
    }

    info!("Goodbye.");
    Ok(())
}

/// Block-reactive scan loop for Aave V3 pools.
async fn run_aave_chain(
    chain: ChainConfig,
    private_key: String,
    dry_run: bool,
    telegram_bot_token: Option<String>,
    telegram_chat_id: Option<String>,
    dashboard: SharedDashboard,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    let chain_name = chain.name.clone();
    let min_profit = chain.min_profit_usd;
    let liquidator_contract = chain.liquidator_contract;
    let block_poll_ms = chain.poll_interval_ms;
    let full_scan_interval = std::time::Duration::from_millis(chain.scan_interval_ms);
    let gas_token_symbol = chain.gas_token_symbol.clone();
    let is_v2 = chain.is_v2;

    let wss_url = chain.wss_url.clone();
    info!(chain = %chain_name, rpc = %chain.rpc_url, "Connecting...");

    let signer: PrivateKeySigner = private_key.parse()?;
    let signer_address = signer.address();

    // Try WSS first (lower latency), fallback to HTTP
    let connect_url = if let Some(ref wss) = wss_url {
        info!(chain = %chain_name, "Trying WSS connection...");
        wss.clone()
    } else {
        chain.rpc_url.clone()
    };

    let provider = match ProviderBuilder::new()
        .wallet(signer.clone())
        .connect(&connect_url)
        .await
    {
        Ok(p) => p,
        Err(e) if wss_url.is_some() => {
            warn!(chain = %chain_name, "WSS failed ({e}), falling back to HTTP");
            ProviderBuilder::new()
                .wallet(signer)
                .connect(&chain.rpc_url)
                .await?
        }
        Err(e) => return Err(e.into()),
    };

    let block = provider.get_block_number().await?;
    let transport = if connect_url.starts_with("wss") { "WSS" } else { "HTTP" };
    info!(chain = %chain_name, block, transport, "Connected");

    // Push startup event
    {
        let mut state = dashboard.write().await;
        state.push_event(DashboardEvent {
            timestamp_unix: now_unix(),
            chain: chain_name.clone(),
            event_type: "info".to_string(),
            message: format!("Connected at block {block}"),
            data: None,
        });
    }

    let chain_id = chain.chain_id;
    let cross_token_map = chain.cross_token_map.clone();
    let mut monitor = AaveV3Monitor::new(chain);
    monitor.init(&provider).await?;
    let mut executor = AaveV3Executor::new(
        chain_name.clone(),
        chain_id,
        liquidator_contract,
        dry_run,
        min_profit,
        telegram_bot_token,
        telegram_chat_id,
        dashboard.clone(),
        cross_token_map,
    );

    // Query initial wallet balance
    if let Ok(bal) = provider.get_balance(signer_address).await {
        let balance_f = u256_to_f64(bal) / 1e18;
        let mut state = dashboard.write().await;
        state.wallet_balances.insert(
            chain_name.clone(),
            crate::core::dashboard::WalletBalance {
                chain: chain_name.clone(),
                symbol: gas_token_symbol.clone(),
                balance: balance_f,
                balance_usd: 0.0,
            },
        );
    }

    // Phase 6.1: Try newHeads subscription via WSS (<100ms latency vs 3s polling)
    let mut block_stream = match provider.subscribe_blocks().await {
        Ok(sub) => {
            info!(chain = %chain_name, "newHeads subscription active (<100ms block latency)");
            {
                let mut state = dashboard.write().await;
                state.push_event(DashboardEvent {
                    timestamp_unix: now_unix(),
                    chain: chain_name.clone(),
                    event_type: "info".to_string(),
                    message: "newHeads subscription active".to_string(),
                    data: None,
                });
            }
            Some(sub.into_stream())
        }
        Err(e) => {
            info!(chain = %chain_name, "newHeads unavailable ({e}), polling every {block_poll_ms}ms");
            None
        }
    };

    info!(
        chain = %chain_name,
        block_poll_ms,
        full_scan_s = full_scan_interval.as_secs(),
        at_risk = monitor.at_risk_count(),
        mode = if block_stream.is_some() { "subscription" } else { "polling" },
        "Starting scan loop"
    );

    let mut consecutive_errors = 0u32;
    let mut last_block = block;
    let mut last_full_scan = std::time::Instant::now();
    let mut stats_timer = tokio::time::Instant::now();
    let poll_duration = std::time::Duration::from_millis(block_poll_ms);

    loop {
        if *shutdown_rx.borrow() {
            info!(chain = %chain_name, "Shutdown received");
            break;
        }

        let current_block;

        if let Some(ref mut stream) = block_stream {
            // Subscription-based: instant notification on new block
            tokio::select! {
                header_opt = stream.next() => {
                    match header_opt {
                        Some(header) => {
                            current_block = header.number;
                            consecutive_errors = 0;
                        }
                        None => {
                            // Stream ended — WSS disconnected, fall back to polling
                            warn!(chain = %chain_name, "newHeads stream ended, falling back to polling");
                            block_stream = None;
                            continue;
                        }
                    }
                }
                _ = shutdown_rx.changed() => break,
            }
        } else {
            // Polling fallback
            current_block = match provider.get_block_number().await {
                Ok(b) => {
                    consecutive_errors = 0;
                    b
                }
                Err(e) => {
                    consecutive_errors += 1;
                    if consecutive_errors % 10 == 1 {
                        warn!(chain = %chain_name, "Block poll failed: {e}");
                    }
                    tokio::time::sleep(poll_duration).await;
                    continue;
                }
            };

            if current_block <= last_block {
                tokio::select! {
                    _ = tokio::time::sleep(poll_duration) => {}
                    _ = shutdown_rx.changed() => break,
                }
                continue;
            }
        }

        // Skip duplicate blocks (possible with subscription on fast chains)
        if current_block <= last_block {
            continue;
        }

        let blocks_advanced = current_block - last_block;
        if blocks_advanced > 1 {
            debug!(chain = %chain_name, block = current_block, skipped = blocks_advanced - 1, "Blocks advanced");
        }
        last_block = current_block;

        // Quick scan
        match monitor.quick_scan(&provider).await {
            Ok(candidates) => {
                if !candidates.is_empty() {
                    info!(chain = %chain_name, block = current_block, count = candidates.len(), "Quick scan found candidates!");
                    executor.process_candidates(&provider, signer_address, &candidates).await;
                }
            }
            Err(e) => {
                warn!(chain = %chain_name, "Quick scan error: {e}");
            }
        }

        // Incremental discovery
        if let Err(e) = monitor.incremental_discover(&provider, current_block).await {
            debug!(chain = %chain_name, "Incremental discover error: {e}");
        }

        // Full scan
        if last_full_scan.elapsed() >= full_scan_interval {
            let scan_rpc_ok;
            match monitor.full_scan(&provider).await {
                Ok(candidates) => {
                    scan_rpc_ok = true;
                    if !candidates.is_empty() {
                        executor.process_candidates(&provider, signer_address, &candidates).await;
                    }
                    executor.clean_cooldowns();
                }
                Err(e) => {
                    scan_rpc_ok = false;
                    error!(chain = %chain_name, "Full scan error: {e}");
                    let mut state = dashboard.write().await;
                    state.push_event(DashboardEvent {
                        timestamp_unix: now_unix(),
                        chain: chain_name.clone(),
                        event_type: "error".to_string(),
                        message: format!("Scan error: {}", &e.to_string()[..e.to_string().len().min(80)]),
                        data: None,
                    });
                }
            }
            last_full_scan = std::time::Instant::now();

            // Update dashboard after EVERY full scan
            let thresholds = monitor.compute_liquidation_thresholds();
            let at_risk_snaps: Vec<AtRiskSnapshot> = thresholds
                .iter()
                .take(20)
                .map(|(addr, symbol, pct)| AtRiskSnapshot {
                    address: format!("{addr}"),
                    health_factor: 1.0 / (1.0 - pct / 100.0),
                    debt_usd: 0.0,
                    collateral_symbol: symbol.clone(),
                    drop_needed_pct: *pct,
                })
                .collect();

            let protocol_name = if is_v2 { "Aave V2" } else { "Aave V3" };
            {
                let mut state = dashboard.write().await;
                state.update_monitor(&chain_name, MonitorSnapshot {
                    chain: chain_name.clone(),
                    protocol: protocol_name.to_string(),
                    borrowers: monitor.borrower_count(),
                    positions: monitor.position_count(),
                    at_risk: monitor.at_risk_count(),
                    last_scan_ms: monitor.last_scan_duration_ms(),
                    total_scans: monitor.total_scans(),
                    last_block: current_block,
                    rpc_ok: scan_rpc_ok,
                    at_risk_positions: at_risk_snaps,
                    updated_at_unix: now_unix(),
                });
                state.update_stats(&chain_name, executor.stats());
            }
        }

        // Wallet balance + detailed stats every 5 min
        if stats_timer.elapsed() >= std::time::Duration::from_secs(300) {
            let stats = executor.stats();
            info!(
                chain = %chain_name,
                block = current_block,
                borrowers = monitor.borrower_count(),
                at_risk = monitor.at_risk_count(),
                scans = monitor.total_scans(),
                candidates = stats.total_candidates,
                success = stats.total_success,
                profit_usd = format!("${:.2}", stats.total_profit_usd),
                "STATS"
            );

            // Query wallet balance
            if let Ok(bal) = provider.get_balance(signer_address).await {
                let balance_f = u256_to_f64(bal) / 1e18;
                let mut state = dashboard.write().await;
                state.wallet_balances.insert(
                    chain_name.clone(),
                    crate::core::dashboard::WalletBalance {
                        chain: chain_name.clone(),
                        symbol: gas_token_symbol.clone(),
                        balance: balance_f,
                        balance_usd: 0.0, // TODO: price oracle
                    },
                );
            }

            stats_timer = tokio::time::Instant::now();
        }
    }

    Ok(())
}

/// Block-reactive scan loop for Morpho Blue.
async fn run_morpho_chain(
    config: MorphoConfig,
    rpc_url: String,
    wss_url: Option<String>,
    private_key: String,
    dry_run: bool,
    liquidator_contract: Option<alloy::primitives::Address>,
    min_profit_usd: f64,
    dashboard: SharedDashboard,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    let chain_name = config.chain_name.clone();
    let poll_ms = config.poll_interval_ms;
    let full_scan_interval = std::time::Duration::from_millis(config.scan_interval_ms);

    info!(chain = %chain_name, rpc = %rpc_url, "Connecting Morpho...");

    let signer: PrivateKeySigner = private_key.parse()?;
    let signer_address = signer.address();

    // Try WSS first (lower latency + subscriptions), fallback to HTTP
    let connect_url = if let Some(ref wss) = wss_url {
        wss.clone()
    } else {
        rpc_url.clone()
    };

    let provider = match ProviderBuilder::new()
        .wallet(signer.clone())
        .connect(&connect_url)
        .await
    {
        Ok(p) => p,
        Err(e) if wss_url.is_some() => {
            warn!(chain = %chain_name, "WSS failed ({e}), falling back to HTTP");
            ProviderBuilder::new()
                .wallet(signer)
                .connect(&rpc_url)
                .await?
        }
        Err(e) => return Err(e.into()),
    };

    let block = provider.get_block_number().await?;
    let transport = if connect_url.starts_with("wss") { "WSS" } else { "HTTP" };
    info!(chain = %chain_name, block, transport, "Morpho connected");

    {
        let mut state = dashboard.write().await;
        state.push_event(DashboardEvent {
            timestamp_unix: now_unix(),
            chain: chain_name.clone(),
            event_type: "info".to_string(),
            message: format!("Morpho connected at block {block}"),
            data: None,
        });
    }

    let mut monitor = MorphoMonitor::new(config);
    monitor.init(&provider).await?;

    let mut executor = MorphoExecutor::new(
        chain_name.clone(),
        liquidator_contract,
        dry_run,
        min_profit_usd,
        dashboard.clone(),
    );

    // Phase 6.1: Try newHeads subscription
    let mut block_stream = match provider.subscribe_blocks().await {
        Ok(sub) => {
            info!(chain = %chain_name, "Morpho newHeads subscription active");
            Some(sub.into_stream())
        }
        Err(e) => {
            info!(chain = %chain_name, "Morpho newHeads unavailable ({e}), polling every {poll_ms}ms");
            None
        }
    };

    let poll_duration = std::time::Duration::from_millis(poll_ms);
    let mut last_block = block;
    let mut last_full_scan = std::time::Instant::now();

    loop {
        if *shutdown_rx.borrow() {
            info!(chain = %chain_name, "Morpho shutdown");
            break;
        }

        let current_block;

        if let Some(ref mut stream) = block_stream {
            tokio::select! {
                header_opt = stream.next() => {
                    match header_opt {
                        Some(header) => {
                            current_block = header.number;
                        }
                        None => {
                            warn!(chain = %chain_name, "Morpho newHeads stream ended, falling back to polling");
                            block_stream = None;
                            continue;
                        }
                    }
                }
                _ = shutdown_rx.changed() => break,
            }
        } else {
            current_block = match provider.get_block_number().await {
                Ok(b) => b,
                Err(_) => {
                    tokio::time::sleep(poll_duration).await;
                    continue;
                }
            };

            if current_block <= last_block {
                tokio::select! {
                    _ = tokio::time::sleep(poll_duration) => {}
                    _ = shutdown_rx.changed() => break,
                }
                continue;
            }
        }

        if current_block <= last_block {
            continue;
        }
        last_block = current_block;

        // Quick scan
        match monitor.quick_scan(&provider).await {
            Ok(candidates) if !candidates.is_empty() => {
                executor.process_candidates(&provider, signer_address, &candidates).await;
            }
            _ => {}
        }

        // Incremental discovery
        let _ = monitor.incremental_discover(&provider, current_block).await;

        // Full scan
        if last_full_scan.elapsed() >= full_scan_interval {
            match monitor.full_scan(&provider).await {
                Ok(candidates) if !candidates.is_empty() => {
                    executor.process_candidates(&provider, signer_address, &candidates).await;
                }
                Err(e) => {
                    error!(chain = %chain_name, "Morpho full scan error: {e}");
                }
                _ => {}
            }
            executor.clean_cooldowns();
            last_full_scan = std::time::Instant::now();

            // Update dashboard after every scan
            let at_risk_data = monitor.at_risk_snapshots();
            let at_risk_snaps: Vec<AtRiskSnapshot> = at_risk_data.iter().take(20).map(|(addr, hf, debt, sym)| {
                AtRiskSnapshot {
                    address: format!("{addr}"),
                    health_factor: *hf,
                    debt_usd: *debt,
                    collateral_symbol: sym.clone(),
                    drop_needed_pct: if *hf > 0.0 { (1.0 - 1.0 / hf) * 100.0 } else { 0.0 },
                }
            }).collect();

            {
                let mut state = dashboard.write().await;
                state.update_monitor(&chain_name, MonitorSnapshot {
                    chain: chain_name.clone(),
                    protocol: "Morpho Blue".to_string(),
                    borrowers: monitor.borrower_count(),
                    positions: monitor.position_count(),
                    at_risk: monitor.at_risk_count(),
                    last_scan_ms: monitor.last_scan_duration_ms(),
                    total_scans: monitor.total_scans(),
                    last_block: current_block,
                    rpc_ok: true,
                    at_risk_positions: at_risk_snaps,
                    updated_at_unix: now_unix(),
                });
                state.update_stats(&chain_name, executor.stats());
            }
        }
    }

    Ok(())
}
