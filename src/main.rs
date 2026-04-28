mod api;
mod cache;
mod config;
mod expression;
mod modbus;
mod models;
mod poller_manager;
mod storage;

use api::AppState;
use cache::DeviceCache;
use poller_manager::PollerManager;
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
        "Loaded config: {} devices in YAML, port {}",
        app_config.devices.len(),
        app_config.server_port
    );

    let cache = DeviceCache::new();
    let storage = Storage::new(&app_config.db_path)?;

    // Auto-seed on startup if no admin user exists
    storage.seed_database().await?;

    // Migrate YAML devices to DB (one-time)
    if !storage.is_yaml_migrated().await? && !app_config.devices.is_empty() {
        storage.import_yaml_devices(&app_config.devices).await?;
        storage.mark_yaml_migrated().await?;
        tracing::info!("Migrated {} devices from YAML to database", app_config.devices.len());
    }

    // Load device configs from DB and start pollers
    let device_configs = storage.get_all_device_configs().await?;
    let poller_manager = PollerManager::new(cache.clone());
    for config in &device_configs {
        poller_manager.start(config.clone()).await;
        tracing::info!("Started poller for device {}", config.id);
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

                // Evaluate alarm thresholds
                if let Ok(sources) = storage.list_data_sources(None).await {
                    let mut value_map = std::collections::HashMap::new();
                    for snap in &snapshots {
                        for (metric, value) in &snap.metrics {
                            value_map.insert(
                                format!("{}:{}", snap.device_id, metric),
                                *value,
                            );
                        }
                    }

                    for source in &sources {
                        let value = if source.source_type == "modbus" {
                            *value_map
                                .get(&format!("{}:{}", source.device_id, source.key))
                                .unwrap_or(&source.base_val)
                        } else {
                            continue;
                        };

                        if source.crit_val > 0.0 && value >= source.crit_val {
                            if !storage.has_active_alarm(&source.key, "critical").await.unwrap_or(false) {
                                let msg = format!(
                                    "{} reached {}{} (critical: {}{})",
                                    source.name, value, source.unit, source.crit_val, source.unit
                                );
                                let _ = storage.create_alarm(&source.key, "critical", value, source.crit_val, &msg).await;
                            }
                        } else if source.warn_val > 0.0 && value >= source.warn_val {
                            if !storage.has_active_alarm(&source.key, "warning").await.unwrap_or(false) {
                                let msg = format!(
                                    "{} reached {}{} (warning: {}{})",
                                    source.name, value, source.unit, source.warn_val, source.unit
                                );
                                let _ = storage.create_alarm(&source.key, "warning", value, source.warn_val, &msg).await;
                            }
                        } else {
                            let _ = storage.resolve_active_alarms(&source.key).await;
                        }
                    }
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
        poller_manager,
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
