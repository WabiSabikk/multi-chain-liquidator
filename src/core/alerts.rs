use crate::core::types::LiquidationCandidate;
use tracing::warn;

/// Send a Telegram notification about a liquidation event
pub async fn send_telegram_alert(
    bot_token: &str,
    chat_id: &str,
    chain_name: &str,
    candidate: &LiquidationCandidate,
    status: &str,
    tx_hash: Option<&str>,
) {
    let emoji = match status {
        "SUCCESS" => "\u{1F4B0}",
        "REVERTED" => "\u{274C}",
        "DRY_RUN" => "\u{1F441}",
        _ => "\u{26A0}\u{FE0F}",
    };

    let mut text = format!(
        "{emoji} *[{chain_name}] {status}*\n\
         Target: `{addr}`\n\
         HF: {hf:.4} | eMode: {emode}\n\
         {debt_sym} -> {col_sym}\n\
         Debt: ${debt:.0}\n\
         Est. Profit: ${profit:.2}",
        addr = &format!("{}", candidate.address)[..14],
        hf = candidate.health_factor,
        emode = candidate.e_mode_category,
        debt_sym = candidate.debt_symbol,
        col_sym = candidate.collateral_symbol,
        debt = candidate.total_debt_usd,
        profit = candidate.estimated_profit_usd,
    );

    if let Some(hash) = tx_hash {
        text.push_str(&format!("\nTX: `{hash}`"));
    }

    let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown",
        }))
        .send()
        .await;

    if let Err(e) = res {
        warn!("Telegram alert failed: {e}");
    }
}
