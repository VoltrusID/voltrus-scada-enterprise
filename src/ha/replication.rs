// Replication protocol — stub for compilation
// Full implementation in Task 7

use crate::ha::HaState;
use crate::models::HaConfig;

pub async fn run_replication_server(_config: &HaConfig, _state: HaState) -> anyhow::Result<()> {
    tracing::info!("Replication server stub — not yet implemented");
    Ok(())
}

pub async fn run_replication_client(_config: &HaConfig, _state: HaState) -> anyhow::Result<()> {
    tracing::info!("Replication client stub — not yet implemented");
    Ok(())
}
