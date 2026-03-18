use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Json};
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::core::types::ExecutorStats;

pub type SharedDashboard = Arc<RwLock<DashboardState>>;

pub fn new_shared_dashboard() -> SharedDashboard {
    Arc::new(RwLock::new(DashboardState::new()))
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardState {
    pub started_at_unix: u64,
    pub dry_run: bool,
    pub wallet_address: String,
    pub wallet_balances: HashMap<String, WalletBalance>,
    pub monitors: HashMap<String, MonitorSnapshot>,
    pub events: VecDeque<DashboardEvent>,
    pub executor_stats: HashMap<String, ExecutorStats>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WalletBalance {
    pub chain: String,
    pub symbol: String,
    pub balance: f64,
    pub balance_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonitorSnapshot {
    pub chain: String,
    pub protocol: String,
    pub borrowers: usize,
    pub positions: usize,
    pub at_risk: usize,
    pub last_scan_ms: u64,
    pub total_scans: u64,
    pub last_block: u64,
    pub rpc_ok: bool,
    pub at_risk_positions: Vec<AtRiskSnapshot>,
    pub updated_at_unix: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AtRiskSnapshot {
    pub address: String,
    pub health_factor: f64,
    pub debt_usd: f64,
    pub collateral_symbol: String,
    pub drop_needed_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardEvent {
    pub timestamp_unix: u64,
    pub chain: String,
    pub event_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl DashboardState {
    pub fn new() -> Self {
        Self {
            started_at_unix: now_unix(),
            dry_run: true,
            wallet_address: String::new(),
            wallet_balances: HashMap::new(),
            monitors: HashMap::new(),
            events: VecDeque::new(),
            executor_stats: HashMap::new(),
        }
    }

    pub fn update_monitor(&mut self, key: &str, snapshot: MonitorSnapshot) {
        self.monitors.insert(key.to_string(), snapshot);
    }

    pub fn push_event(&mut self, event: DashboardEvent) {
        // Persist to events.jsonl for debugging across restarts
        if let Ok(line) = serde_json::to_string(&event) {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("data/events.jsonl")
            {
                let _ = writeln!(f, "{line}");
            }
        }

        self.events.push_front(event);
        while self.events.len() > 500 {
            self.events.pop_back();
        }
    }

    pub fn update_stats(&mut self, chain: &str, stats: &ExecutorStats) {
        self.executor_stats.insert(chain.to_string(), stats.clone());
    }
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

struct AppState {
    dashboard: SharedDashboard,
    token: String,
}

#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

fn check_token(token: &str, query: &TokenQuery) -> bool {
    query.token.as_deref() == Some(token)
}

async fn api_status(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TokenQuery>,
) -> impl IntoResponse {
    if !check_token(&state.token, &query) {
        return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
    }
    let dashboard = state.dashboard.read().await;
    Json(dashboard.clone()).into_response()
}

async fn api_events(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TokenQuery>,
) -> impl IntoResponse {
    if !check_token(&state.token, &query) {
        return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
    }
    let dashboard = state.dashboard.read().await;
    let events: Vec<&DashboardEvent> = dashboard.events.iter().take(100).collect();
    Json(events).into_response()
}

async fn dashboard_html(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TokenQuery>,
) -> impl IntoResponse {
    if !check_token(&state.token, &query) {
        return Html(LOGIN_HTML.to_string()).into_response();
    }
    Html(DASHBOARD_HTML.to_string()).into_response()
}

pub async fn run_dashboard_server(dashboard: SharedDashboard, port: u16, token: String) {
    let state = Arc::new(AppState { dashboard, token });

    let app = Router::new()
        .route("/", get(dashboard_html))
        .route("/api/status", get(api_status))
        .route("/api/events", get(api_events))
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    info!(port, "Dashboard server starting");

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Dashboard bind failed on {addr}: {e}");
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        error!("Dashboard server error: {e}");
    }
}

const DASHBOARD_HTML: &str = include_str!("../../static/dashboard.html");
const LOGIN_HTML: &str = include_str!("../../static/login.html");
