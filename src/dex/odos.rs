use alloy::primitives::{Address, Bytes};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

const ODOS_API: &str = "https://api.odos.xyz";

/// Supported chain IDs for Odos
const SUPPORTED_CHAINS: &[u64] = &[5000]; // Mantle only for now

#[derive(Debug, Clone)]
pub struct OdosRoute {
    pub router: Address,
    pub calldata: Bytes,
    pub amount_out: String,
}

#[derive(Serialize)]
struct InputToken {
    #[serde(rename = "tokenAddress")]
    token_address: String,
    amount: String,
}

#[derive(Serialize)]
struct OutputToken {
    #[serde(rename = "tokenAddress")]
    token_address: String,
    proportion: f64,
}

#[derive(Serialize)]
struct QuoteRequest {
    #[serde(rename = "chainId")]
    chain_id: u64,
    #[serde(rename = "inputTokens")]
    input_tokens: Vec<InputToken>,
    #[serde(rename = "outputTokens")]
    output_tokens: Vec<OutputToken>,
    #[serde(rename = "userAddr")]
    user_addr: String,
    #[serde(rename = "slippageLimitPercent")]
    slippage_limit_percent: f64,
    #[serde(rename = "disableRFQs")]
    disable_rfqs: bool,
    compact: bool,
}

#[derive(Deserialize)]
struct QuoteResponse {
    #[serde(rename = "pathId")]
    path_id: Option<String>,
    #[serde(rename = "outAmounts")]
    out_amounts: Option<Vec<String>>,
}

#[derive(Serialize)]
struct AssembleRequest {
    #[serde(rename = "pathId")]
    path_id: String,
    #[serde(rename = "userAddr")]
    user_addr: String,
}

#[derive(Deserialize)]
struct AssembleResponse {
    transaction: Option<TxData>,
}

#[derive(Deserialize)]
struct TxData {
    to: Option<String>,
    data: Option<String>,
}

pub fn is_supported(chain_id: u64) -> bool {
    SUPPORTED_CHAINS.contains(&chain_id)
}

/// Get swap route from Odos aggregator.
/// `contract_addr` is our liquidator contract that will execute the swap (used as userAddr).
pub async fn get_route(
    chain_id: u64,
    token_in: Address,
    token_out: Address,
    amount_in: alloy::primitives::U256,
    contract_addr: Address,
) -> Option<OdosRoute> {
    if !is_supported(chain_id) {
        return None;
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    // Step 1: Quote
    let quote = QuoteRequest {
        chain_id,
        input_tokens: vec![InputToken {
            token_address: format!("{token_in:#x}"),
            amount: amount_in.to_string(),
        }],
        output_tokens: vec![OutputToken {
            token_address: format!("{token_out:#x}"),
            proportion: 1.0,
        }],
        user_addr: format!("{contract_addr:#x}"),
        slippage_limit_percent: 2.0,
        disable_rfqs: true,
        compact: true,
    };

    let resp = client
        .post(format!("{ODOS_API}/sor/quote/v2"))
        .json(&quote)
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        warn!(chain_id, "Odos quote failed: HTTP {}", resp.status());
        return None;
    }

    let quote_resp: QuoteResponse = resp.json().await.ok()?;
    let path_id = quote_resp.path_id?;
    let amount_out = quote_resp
        .out_amounts
        .and_then(|v| v.into_iter().next())
        .unwrap_or_default();

    // Step 2: Assemble
    let asm = AssembleRequest {
        path_id,
        user_addr: format!("{contract_addr:#x}"),
    };

    let resp2 = client
        .post(format!("{ODOS_API}/sor/assemble"))
        .json(&asm)
        .send()
        .await
        .ok()?;

    if !resp2.status().is_success() {
        warn!(chain_id, "Odos assemble failed: HTTP {}", resp2.status());
        return None;
    }

    let asm_resp: AssembleResponse = resp2.json().await.ok()?;
    let tx = asm_resp.transaction?;
    let router_str = tx.to?;
    let calldata_hex = tx.data?;

    let router: Address = router_str.parse().ok()?;
    let calldata = Bytes::from(hex::decode(calldata_hex.trim_start_matches("0x")).ok()?);

    info!(
        chain_id,
        router = %router,
        calldata_len = calldata.len(),
        amount_out = %amount_out,
        "Odos route found"
    );

    Some(OdosRoute {
        router,
        calldata,
        amount_out,
    })
}
