// macro allowing us to convert human readable units to workspace units.
use near_units::parse_near;

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;

// Additional convenient imports that allows workspaces to function readily.
use workspaces::prelude::*;

const WASM_FILEPATH: &str = "./target/debug/atomic_swap.wasm";

#[tokio::test]
async fn test_atomic_swap_contract() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let wasm = std::fs::read(WASM_FILEPATH)?;
    let contract = worker.dev_deploy(wasm).await?;

    Ok(())
}