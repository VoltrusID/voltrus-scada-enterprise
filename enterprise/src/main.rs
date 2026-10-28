use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    voltrus::run(Arc::new(voltrus_enterprise::Enterprise)).await
}
