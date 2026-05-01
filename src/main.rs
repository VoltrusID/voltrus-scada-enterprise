mod api;
mod auth;
mod cache;
mod config;
mod expression;
mod flow_engine;
mod modbus;
mod models;
mod poller_manager;
mod retention;
mod security;
mod storage;
mod timeutil;

use api::AppState;
use cache::DeviceCache;
use models::DeviceStatus;
use poller_manager::PollerManager;
use std::sync::Arc;
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

    let flow_engine = crate::flow_engine::engine::FlowEngine::new(
        storage.clone(),
        cache.clone(),
    );
    if let Ok(flows) = storage.list_flows().await {
        for flow in flows {
            if flow.enabled {
                match (serde_json::from_str::<crate::flow_engine::trigger::TriggerConfig>(&flow.trigger_config),
                       serde_json::from_str::<Vec<crate::flow_engine::node::NodeConfig>>(&flow.nodes)) {
                    (Ok(trigger), Ok(nodes)) => {
                        let engine_flow = crate::flow_engine::engine::Flow {
                            id: flow.id,
                            name: flow.name,
                            enabled: flow.enabled,
                            trigger,
                            nodes,
                        };
                        flow_engine.add_flow(engine_flow).await;
                    }
                    _ => {
                        tracing::warn!("Failed to deserialize flow {} config, skipping", flow.id);
                    }
                }
            }
        }
    }
    flow_engine.spawn().await;

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
                            // Find device metrics
                            let metrics: std::collections::HashMap<String, f64> = snapshots
                                .iter()
                                .filter(|s| s.device_id == source.device_id && s.status == DeviceStatus::Online)
                                .flat_map(|s| s.metrics.clone())
                                .collect();

                            if metrics.is_empty() {
                                continue;
                            }

                            // Check for expression in source_config
                            if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&source.source_config) {
                                if let Some(expr_str) = cfg.get("expression").and_then(|v| v.as_str()) {
                                    match expression::evaluate(expr_str, &metrics) {
                                        Ok(val) => val,
                                        Err(_) => continue,
                                    }
                                } else {
                                    *value_map
                                        .get(&format!("{}:{}", source.device_id, source.key))
                                        .unwrap_or(&source.base_val)
                                }
                            } else {
                                *value_map
                                    .get(&format!("{}:{}", source.device_id, source.key))
                                    .unwrap_or(&source.base_val)
                            }
                        } else {
                            continue;
                        };

                        if source.crit_val > 0.0 && value >= source.crit_val {
                            let has_active = storage.has_active_alarm(&source.key, "critical").await.unwrap_or(false);
                            let in_cooldown = storage.is_in_cooldown(&source.key, "critical").await.unwrap_or(false);
                            if !has_active && !in_cooldown {
                                let msg = format!(
                                    "{} reached {}{} (critical: {}{})",
                                    source.name, value, source.unit, source.crit_val, source.unit
                                );
                                let _ = storage.create_alarm(&source.key, "critical", value, source.crit_val, &msg).await;
                            }
                        } else if source.warn_val > 0.0 && value >= source.warn_val {
                            let has_active = storage.has_active_alarm(&source.key, "warning").await.unwrap_or(false);
                            let in_cooldown = storage.is_in_cooldown(&source.key, "warning").await.unwrap_or(false);
                            if !has_active && !in_cooldown {
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

    // Spawn retention task
    {
        let storage = storage.clone();
        let retention_config = app_config.retention.clone();
        tokio::spawn(async move {
            retention::run_retention_task(storage, retention_config).await;
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

    let oidc_config = auth::oidc::OidcConfig {
        enabled: app_config.oidc_enabled,
        issuer: app_config.oidc_issuer.clone(),
        audience: app_config.oidc_audience.clone(),
        jwks_url: app_config.oidc_jwks_url.clone(),
        role_claim: app_config.oidc_role_claim.clone().into(),
        admin_role: app_config.oidc_admin_role.clone().into(),
    };

    let jwks_manager = auth::oidc::JwksManager::new(&oidc_config.jwks_url);
    if oidc_config.enabled {
        auth::oidc::spawn_jwks_refresh(jwks_manager.clone());
        tracing::info!("OIDC enabled, issuer: {}", oidc_config.issuer);
    }

    let login_rate_limiter = crate::security::LoginRateLimiter::new();
    let (audit_logger, mut audit_rx) = crate::security::AuditLogger::new();

    // Spawn audit log consumer
    {
        let storage = storage.clone();
        tokio::spawn(async move {
            while let Some(event) = audit_rx.recv().await {
                if let Err(e) = storage.log_audit_event(
                    &event.action,
                    &event.username,
                    &event.resource,
                    event.details.as_deref(),
                ).await {
                    tracing::warn!("Failed to write audit log: {}", e);
                }
            }
        });
    }

    let state = AppState {
        cache: cache.clone(),
        storage: storage.clone(),
        poller_manager,
        started_at: std::time::Instant::now(),
        auth_secret,
        cors_origin: app_config.cors_origin.clone(),
        cookie_secure: app_config.cookie_secure,
        oidc_config,
        jwks_manager,
        retention_config: app_config.retention.clone(),
        flow_engine: Some(Arc::new(flow_engine)),
        login_rate_limiter,
        audit_logger,
    };

    api::stream::spawn_sse_composer(cache, storage);
    let app = api::create_router(state);

    let addr = format!("0.0.0.0:{}", app_config.server_port);
    tracing::info!("Server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        tracing::info!("Received shutdown signal, draining connections...");

        // Give connections 3s to drain, then force quit
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        tracing::info!("Drain timeout, forcing shutdown");
        std::process::exit(0);
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}
