#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod ai_copilot;
mod analytics;
mod api;
mod asset;
mod auth;
mod backup;
mod bacnet;
mod cache;
mod cli;
mod command_queue;
mod compliance;
mod compression;
mod config;
mod config_export;
mod deadband;
mod discovery;
mod dnp3;
mod edr_compat;
mod ethernet_ip;
mod expression;
mod flow_engine;
mod ha;
mod hardware_cert;
mod integration;
mod hub;
mod import_nodered;
mod license;
mod modbus;
mod models;
mod mqtt_broker;
mod network;
mod notification;
mod oee;
mod opcua;
mod poller_manager;
mod recipe;
mod report;
mod report_scheduler;
mod retention;
mod s7;
mod script_engine;
mod security;
mod sim_engine;
mod storage;
mod timeutil;
mod video;
mod widget_sdk;
mod ws_manager;
mod ws_types;

use api::AppState;
use cache::DeviceCache;
use models::DeviceStatus;
use poller_manager::PollerManager;
use std::sync::Arc;
use storage::Storage;

fn run_watchdog() {
    tracing_subscriber::fmt::init();
    tracing::info!("Voltrus watchdog process started");

    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    rt.block_on(async {
        let mut failures = 0u32;
        let max_failures = 3u32;
        let max_restarts = 5u32;
        let mut restarts = 0u32;
        let cooldown = std::time::Duration::from_secs(30);

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));

        loop {
            interval.tick().await;

            let healthy = matches!(
                reqwest::get("http://localhost:8080/api/v1/health").await,
                Ok(resp) if resp.status().is_success()
            );

            if healthy {
                failures = 0;
                continue;
            }

            failures += 1;
            tracing::warn!("Health check failed ({}/{})", failures, max_failures);

            if failures >= max_failures && restarts < max_restarts {
                tracing::warn!(
                    "Restarting Voltrus main process (attempt {}/{})",
                    restarts + 1,
                    max_restarts
                );

                #[cfg(unix)]
                {
                    let _ = std::process::Command::new("pkill")
                        .args(["-f", "voltrus"])
                        .output();
                }

                tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                let exe = std::env::current_exe().expect("Cannot find executable");
                match std::process::Command::new(exe).spawn() {
                    Ok(child) => {
                        tracing::info!("Started Voltrus process (PID: {:?})", child.id());
                        restarts += 1;
                        failures = 0;
                        tokio::time::sleep(cooldown).await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to start Voltrus: {}", e);
                    }
                }
            }
        }
    });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--watchdog".to_string()) {
        run_watchdog();
        return Ok(());
    }

    if cli::handle_cli(&args).await {
        return Ok(());
    }

    let config_path = args.get(1).map(|s| s.as_str()).unwrap_or("config.yaml");
    let app_config = config::load_config(config_path)?;

    tracing::info!(
        "Loaded config: {} devices in YAML, port {}",
        app_config.devices.len(),
        app_config.server_port
    );

    let cache = DeviceCache::new();
    let ws_manager = Arc::new(crate::ws_manager::WsManager::new());
    let cache = cache.with_ws_manager(ws_manager.clone());
    let storage =
        Storage::new_with_key(&app_config.db_path, app_config.db_encryption_key.as_deref())?;
    let cache = cache.with_storage(storage.clone());
    let license_status = license::resolve_license_status(&storage).await;
    let license_tier = license_status.tier();
    let feature_flags = license_status.feature_flags();

    if !license_status.is_valid() {
        tracing::warn!(
            "License status: {:?} — skipping Modbus polling",
            license_status
        );
    }

    let flow_engine = crate::flow_engine::engine::FlowEngine::new(storage.clone(), cache.clone());
    if let Ok(flows) = storage.list_flows().await {
        for flow in flows {
            if flow.enabled {
                match (
                    serde_json::from_str::<crate::flow_engine::trigger::TriggerConfig>(
                        &flow.trigger_config,
                    ),
                    serde_json::from_str::<Vec<crate::flow_engine::node::NodeConfig>>(&flow.nodes),
                ) {
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
        tracing::info!(
            "Migrated {} devices from YAML to database",
            app_config.devices.len()
        );
    }

    // Load device configs from DB and start pollers (only with valid license)
    let poller_manager = PollerManager::new(
        cache.clone(),
        storage.clone(),
        network::NetworkManager::new(app_config.networks.clone()),
    );
    if license_status.is_valid() {
        let device_configs = storage.get_all_device_configs().await?;
        let device_limit = license_tier.max_devices();
        let mut started_count: u32 = 0;
        for config in &device_configs {
            if let Some(limit) = device_limit
                && started_count >= limit
            {
                tracing::warn!(
                    "Device '{}' skipped — {} tier limit of {} devices reached. Upgrade at https://voltrus.id",
                    config.id,
                    serde_json::to_string(&license_tier).unwrap_or_else(|_| "unknown".to_string()),
                    limit
                );
                continue;
            }
            poller_manager.start(config.clone()).await;
            tracing::info!("Started poller for device {}", config.id);
            started_count += 1;
        }
    } else {
        tracing::warn!("No valid license — Modbus polling disabled");
    }

    // Start OPC-UA pollers (Professional tier only)
    {
        if license_tier.opcua_enabled() {
            match storage.get_all_opcua_device_configs().await {
                Ok(opcua_configs) => {
                    for config in &opcua_configs {
                        poller_manager.start_opcua(config.clone()).await;
                        tracing::info!("Started OPC-UA poller for device {}", config.id);
                    }
                    if !opcua_configs.is_empty() {
                        tracing::info!("Started {} OPC-UA device(s)", opcua_configs.len());
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to load OPC-UA device configs: {}", e);
                }
            }
        } else if let Ok(configs) = storage.get_all_opcua_device_configs().await
            && !configs.is_empty()
        {
            tracing::warn!(
                "OPC-UA devices configured but Professional tier required. Upgrade at https://voltrus.id"
            );
        }
    }

    // Start hub site connectors (Professional tier only)
    let hub_manager = {
        if license_tier.multi_site() {
            let hm = crate::api::hub::HubManager::new();
            match storage.get_enabled_hub_sites().await {
                Ok(sites) => {
                    for site in &sites {
                        hm.start_site(site.clone(), cache.clone(), storage.clone())
                            .await;
                        tracing::info!(
                            "Started hub connector for site '{}' (id={})",
                            site.name,
                            site.id
                        );
                    }
                    if !sites.is_empty() {
                        tracing::info!("Started {} hub site connector(s)", sites.len());
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to load hub sites: {}", e);
                }
            }
            Some(hm)
        } else {
            None
        }
    };

    // Build optional email notifier from SMTP config
    let email_notifier = match &app_config.smtp {
        Some(smtp_cfg) => match notification::email::EmailNotifier::new(smtp_cfg) {
            Ok(notifier) => {
                tracing::info!(
                    "Email notifier initialized (from: {})",
                    smtp_cfg.from_address
                );
                Some(notifier)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize email notifier: {}", e);
                None
            }
        },
        None => None,
    };

    // Start report scheduler (Professional tier, SMTP required)
    if license_tier.pdf_reporting() {
        if let Some(ref notifier) = email_notifier {
            report_scheduler::start_report_scheduler(
                storage.clone(),
                cache.clone(),
                notifier.clone(),
            );
            tracing::info!("PDF report scheduler started");
        } else {
            tracing::info!(
                "PDF reporting enabled but SMTP not configured — report scheduler not started"
            );
        }
    }

    // Build optional push notifier from VAPID config
    let push_notifier = match &app_config.push {
        Some(push_cfg) => match notification::push::PushNotifier::new(push_cfg) {
            Ok(notifier) => {
                tracing::info!("Push notifier initialized");
                Some(notifier)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize push notifier: {}", e);
                None
            }
        },
        None => None,
    };

    // Spawn persistence task
    {
        let cache = cache.clone();
        let storage = storage.clone();
        let email_notifier = email_notifier.clone();
        let push_notifier = push_notifier.clone();
        let alarm_ws_manager = ws_manager.clone();
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
                            value_map.insert(format!("{}:{}", snap.device_id, metric), *value);
                        }
                    }

                    for source in &sources {
                        let value = if source.source_type == "modbus" {
                            // Find device metrics
                            let metrics: std::collections::HashMap<String, f64> = snapshots
                                .iter()
                                .filter(|s| {
                                    s.device_id == source.device_id
                                        && s.status == DeviceStatus::Online
                                })
                                .flat_map(|s| s.metrics.clone())
                                .collect();

                            if metrics.is_empty() {
                                continue;
                            }

                            // Check for expression in source_config
                            if let Ok(cfg) =
                                serde_json::from_str::<serde_json::Value>(&source.source_config)
                            {
                                if let Some(expr_str) =
                                    cfg.get("expression").and_then(|v| v.as_str())
                                {
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

                        // Parse alarm deadband from source configuration
                        let deadband: f64 = if let Ok(cfg) =
                            serde_json::from_str::<serde_json::Value>(&source.source_config)
                        {
                            cfg.get("deadband").and_then(|v| v.as_f64()).unwrap_or(0.0)
                        } else {
                            0.0
                        };

                        if source.crit_val > 0.0 && value >= source.crit_val {
                            // Non-repeating: check for any unresolved alarm (not just un-acked)
                            let has_unresolved = storage
                                .has_unresolved_alarm(&source.key, "critical")
                                .await
                                .unwrap_or(false);
                            let in_cooldown = storage
                                .is_in_cooldown(&source.key, "critical")
                                .await
                                .unwrap_or(false);
                            if !has_unresolved && !in_cooldown {
                                let msg = format!(
                                    "{} reached {}{} (critical: {}{})",
                                    source.name, value, source.unit, source.crit_val, source.unit
                                );
                                match storage
                                    .create_alarm(
                                        &source.key,
                                        "critical",
                                        value,
                                        source.crit_val,
                                        &msg,
                                        deadband,
                                    )
                                    .await
                                {
                                    Ok(alarm_id) => {
                                        let notif_storage = storage.clone();
                                        let notif_notifier = email_notifier.clone();
                                        let notif_push = push_notifier.clone();
                                        let alarm = crate::models::Alarm {
                                            id: alarm_id,
                                            source_key: source.key.clone(),
                                            source_name: source.name.clone(),
                                            severity: "critical".to_string(),
                                            priority: "critical".to_string(),
                                            value,
                                            threshold: source.crit_val,
                                            message: msg.clone(),
                                            triggered_at: crate::timeutil::now_secs(),
                                            resolved_at: None,
                                            acknowledged_at: None,
                                            acknowledged_by: None,
                                            cooldown_until: None,
                                            shelved_until: None,
                                            shelved_by: None,
                                            shelved_reason: None,
                                            deadband,
                                            state: "active".to_string(),
                                        };
                                        // Broadcast alarm via WebSocket
                                        alarm_ws_manager.broadcast_alarm(
                                            crate::ws_types::ServerMessage::Alarm {
                                                alarm_id,
                                                tag_id: source.key.clone(),
                                                severity: "critical".to_string(),
                                                message: msg,
                                                value,
                                                ts: alarm.triggered_at,
                                            },
                                        );
                                        tokio::spawn(async move {
                                            notification::dispatch_alarm_notifications(
                                                notif_storage,
                                                &alarm,
                                                notif_notifier,
                                                notif_push,
                                            )
                                            .await;
                                        });
                                    }
                                    Err(e) => tracing::error!("Failed to create alarm: {}", e),
                                }
                            }
                        } else if source.warn_val > 0.0 && value >= source.warn_val {
                            // Non-repeating: check for any unresolved alarm (not just un-acked)
                            let has_unresolved = storage
                                .has_unresolved_alarm(&source.key, "warning")
                                .await
                                .unwrap_or(false);
                            let in_cooldown = storage
                                .is_in_cooldown(&source.key, "warning")
                                .await
                                .unwrap_or(false);
                            if !has_unresolved && !in_cooldown {
                                let msg = format!(
                                    "{} reached {}{} (warning: {}{})",
                                    source.name, value, source.unit, source.warn_val, source.unit
                                );
                                match storage
                                    .create_alarm(
                                        &source.key,
                                        "warning",
                                        value,
                                        source.warn_val,
                                        &msg,
                                        deadband,
                                    )
                                    .await
                                {
                                    Ok(alarm_id) => {
                                        let notif_storage = storage.clone();
                                        let notif_notifier = email_notifier.clone();
                                        let notif_push = push_notifier.clone();
                                        let alarm = crate::models::Alarm {
                                            id: alarm_id,
                                            source_key: source.key.clone(),
                                            source_name: source.name.clone(),
                                            severity: "warning".to_string(),
                                            priority: "warning".to_string(),
                                            value,
                                            threshold: source.warn_val,
                                            message: msg.clone(),
                                            triggered_at: crate::timeutil::now_secs(),
                                            resolved_at: None,
                                            acknowledged_at: None,
                                            acknowledged_by: None,
                                            cooldown_until: None,
                                            shelved_until: None,
                                            shelved_by: None,
                                            shelved_reason: None,
                                            deadband,
                                            state: "active".to_string(),
                                        };
                                        // Broadcast alarm via WebSocket
                                        alarm_ws_manager.broadcast_alarm(
                                            crate::ws_types::ServerMessage::Alarm {
                                                alarm_id,
                                                tag_id: source.key.clone(),
                                                severity: "warning".to_string(),
                                                message: msg,
                                                value,
                                                ts: alarm.triggered_at,
                                            },
                                        );
                                        tokio::spawn(async move {
                                            notification::dispatch_alarm_notifications(
                                                notif_storage,
                                                &alarm,
                                                notif_notifier,
                                                notif_push,
                                            )
                                            .await;
                                        });
                                    }
                                    Err(e) => tracing::error!("Failed to create alarm: {}", e),
                                }
                            }
                        } else {
                            // Value below thresholds — resolve active alarms with deadband hysteresis.
                            // Only resolves alarms where value < threshold - deadband.
                            // Since deadband defaults to 0, existing behavior is unchanged.
                            let _ = storage
                                .resolve_active_alarms(&source.key, Some(value))
                                .await;
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

    // Spawn alarm auto-unshelf task
    {
        let storage = storage.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = storage.auto_unshelf_expired_alarms().await {
                    tracing::warn!("Auto-unshelf task failed: {}", e);
                }
            }
        });
    }

    // Spawn egress scheduler (v0.39.0, Professional tier+)
    if license_tier.data_egress() {
        let egress_storage = storage.clone();
        let (_egress_shutdown_tx, egress_shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);
        tokio::spawn(async move {
            crate::integration::egress::run_egress_scheduler(egress_storage, egress_shutdown_rx).await;
        });
        tracing::info!("Data egress scheduler started");
    }

    // Spawn OEE monitor (Starter tier and above)
    if license_tier.oee() {
        let oee_cache = cache.clone();
        let oee_storage = storage.clone();
        tokio::spawn(async move {
            oee::run_oee_monitor(oee_cache, oee_storage).await;
        });
        tracing::info!("OEE monitor started");
    }

    // Spawn command queue processor
    {
        let queue = command_queue::CommandQueue::new(storage.clone(), cache.clone())
            .with_ws_manager(ws_manager.clone());
        tokio::spawn(async move {
            queue.run().await;
        });
        tracing::info!("Command queue processor started");
    }

    let auth_secret = match &app_config.auth_secret {
        Some(secret) => {
            anyhow::ensure!(
                secret.len() >= 16,
                "auth_secret must be at least 16 characters"
            );
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
        role_claim: app_config.oidc_role_claim.clone(),
        admin_role: app_config.oidc_admin_role.clone(),
    };

    let jwks_manager = auth::oidc::JwksManager::new(&oidc_config.jwks_url);
    if oidc_config.enabled {
        auth::oidc::spawn_jwks_refresh(jwks_manager.clone());
        tracing::info!("OIDC enabled, issuer: {}", oidc_config.issuer);
    }

    let login_rate_limiter = crate::security::LoginRateLimiter::new();
    let intrusion_detector = crate::security::IntrusionDetector::new();
    let (audit_logger, mut audit_rx) = crate::security::AuditLogger::new();

    // Spawn audit log consumer
    {
        let storage = storage.clone();
        tokio::spawn(async move {
            while let Some(event) = audit_rx.recv().await {
                if let Err(e) = storage
                    .log_audit_event(
                        &event.action,
                        &event.username,
                        &event.resource,
                        event.details.as_deref(),
                    )
                    .await
                {
                    tracing::warn!("Failed to write audit log: {}", e);
                }
            }
        });
    }

    // Initialize MQTT broker (Starter tier and above)
    let mqtt_broker_state = {
        if license_tier.mqtt_broker() {
            let broker_state = crate::mqtt_broker::MqttBrokerState::new();
            let mqtt_cfg = &app_config.mqtt_broker;

            if mqtt_cfg.enabled {
                {
                    let mut cfg = broker_state.config.write().await;
                    *cfg = Some(mqtt_cfg.clone());
                }

                match crate::mqtt_broker::start_broker(broker_state.clone(), mqtt_cfg.clone()).await
                {
                    Ok(()) => {
                        tracing::info!(
                            "Embedded MQTT broker auto-started on port {}",
                            mqtt_cfg.port
                        );
                    }
                    Err(e) => {
                        tracing::warn!("Failed to auto-start MQTT broker: {}", e);
                    }
                }
            }

            Some(broker_state)
        } else {
            None
        }
    };

    // Initialize health monitor and HA state
    let health_monitor = Arc::new(ha::watchdog::HealthMonitor::new(
        app_config.watchdog.clone(),
    ));
    {
        let storage_check = {
            let storage = storage.clone();
            move || match storage.conn_health() {
                true => Ok("ok".to_string()),
                false => Err("connection unhealthy".to_string()),
            }
        };
        health_monitor
            .register("storage", Box::new(storage_check))
            .await;
    }
    if app_config.watchdog.enabled {
        health_monitor.start();
        tracing::info!(
            "Health monitor started (interval: {}s)",
            app_config.watchdog.health_check_interval_secs
        );
    }

    let ha_state = if app_config.ha.enabled {
        let state = ha::HaState::new(&app_config.ha);
        ha::start_ha(&app_config.ha, state.clone());
        tracing::info!(
            "HA enabled, node_id={}, role={}",
            app_config.ha.node_id,
            app_config.ha.role
        );
        state
    } else {
        ha::HaState::disabled()
    };

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
        global_rate_limiter: crate::security::GlobalRateLimiter::new(),
        audit_logger,
        intrusion_detector,
        license_status: license_status.clone(),
        license_tier: license_tier.clone(),
        feature_flags: feature_flags.clone(),
        smtp_config: app_config.smtp.clone(),
        vapid_public_key: app_config.push.as_ref().map(|p| p.vapid_public_key.clone()),
        hub_manager,
        stream_proxy: Some(Arc::new(video::StreamProxy::new("snapshots"))),
        ws_manager: ws_manager.clone(),
        mqtt_broker_state,
        network_manager: network::NetworkManager::new(app_config.networks.clone()),
        health_monitor,
        ha_state,
        widget_registry: widget_sdk::WidgetRegistry::new(std::path::PathBuf::from(
            "voltrus/widgets",
        )),
    };

    api::stream::spawn_sse_composer(cache.clone(), storage.clone());
    crate::ws_manager::spawn_ws_broadcaster(cache, ws_manager);
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
