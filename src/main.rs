mod api;
mod cache;
mod config;
mod modbus;
mod models;
mod storage;

use api::AppState;
use cache::DeviceCache;
use storage::Storage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse()?),
        )
        .init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.yaml".to_string());
    let app_config = config::load_config(&config_path)?;

    tracing::info!(
        "Loaded config: {} devices, port {}",
        app_config.devices.len(),
        app_config.server_port
    );

    let cache = DeviceCache::new();
    let storage = Storage::new(&app_config.db_path)?;

    // Spawn one poller task per device
    for device_config in &app_config.devices {
        let device = device_config.clone();
        let cache = cache.clone();
        tokio::spawn(async move {
            modbus::poll_device(device, cache).await;
        });
        tracing::info!("Started poller for device {}", device_config.id);
    }

    // Spawn persistence task
    {
        let cache = cache.clone();
        let storage = storage.clone();
        let interval = app_config.persist_interval_secs;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval));
            loop {
                ticker.tick().await;
                let snapshots = cache.get_all().await;
                if snapshots.is_empty() {
                    continue;
                }
                let mut records = Vec::new();
                for snap in &snapshots {
                    for (metric, value) in &snap.metrics {
                        records.push((
                            snap.device_id.clone(),
                            snap.timestamp,
                            metric.clone(),
                            *value,
                        ));
                    }
                }
                if let Err(e) = storage.insert_batch(&records).await {
                    tracing::error!("Persistence error: {e}");
                }
            }
        });
    }

    let auth_secret = match &app_config.auth_secret {
        Some(secret) => {
            anyhow::ensure!(secret.len() >= 16, "auth_secret must be at least 16 characters");
            secret.clone()
        }
        None => {
            use rand::Rng;
            let secret: String = rand::rng()
                .sample_iter(&rand::distr::Alphanumeric)
                .take(32)
                .map(char::from)
                .collect();
            tracing::warn!(
                "auth_secret not set in config — generated a random secret. Set auth_secret in config.yaml for persistent sessions."
            );
            secret
        }
    };

    let state = AppState {
        cache,
        storage,
        started_at: std::time::Instant::now(),
        auth_secret,
        cors_origin: app_config.cors_origin.clone(),
        cookie_secure: app_config.cookie_secure,
    };
    let app = api::create_router(state);

    let addr = format!("0.0.0.0:{}", app_config.server_port);
    tracing::info!("Server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        tracing::info!("Received shutdown signal, draining connections...");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}
