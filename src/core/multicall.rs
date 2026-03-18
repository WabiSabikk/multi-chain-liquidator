use alloy::primitives::{Address, Bytes};
use alloy::providers::Provider;
use eyre::Result;
use tracing::{error, warn};

use crate::protocols::aave_v3::contracts::IMulticall3;

/// Single multicall request
#[derive(Debug, Clone)]
pub struct MulticallRequest {
    pub target: Address,
    pub call_data: Bytes,
}

/// Single multicall result
#[derive(Debug, Clone)]
pub struct MulticallResult {
    pub success: bool,
    pub return_data: Bytes,
}

/// Execute batched calls via Multicall3.aggregate3 (staticCall).
/// Splits into chunks of `batch_size` to avoid gas limits.
pub async fn multicall_aggregate<P: Provider + Clone>(
    provider: &P,
    multicall_addr: Address,
    requests: &[MulticallRequest],
    batch_size: usize,
) -> Result<Vec<MulticallResult>> {
    if requests.is_empty() {
        return Ok(vec![]);
    }

    let contract = IMulticall3::new(multicall_addr, provider.clone());
    let mut all_results = Vec::with_capacity(requests.len());

    for chunk in requests.chunks(batch_size) {
        let calls: Vec<IMulticall3::Call3> = chunk
            .iter()
            .map(|req| IMulticall3::Call3 {
                target: req.target,
                allowFailure: true,
                callData: req.call_data.clone(),
            })
            .collect();

        match contract.aggregate3(calls).call().await {
            Ok(results) => {
                // aggregate3 returns Vec<Result> directly (single return)
                for r in results {
                    all_results.push(MulticallResult {
                        success: r.success,
                        return_data: r.returnData,
                    });
                }
            }
            Err(e) => {
                error!("Multicall batch failed: {e}");
                for _ in chunk {
                    all_results.push(MulticallResult {
                        success: false,
                        return_data: Bytes::new(),
                    });
                }
            }
        }
    }

    if all_results.len() != requests.len() {
        warn!(
            "Multicall result count mismatch: got {} expected {}",
            all_results.len(),
            requests.len()
        );
    }

    Ok(all_results)
}
