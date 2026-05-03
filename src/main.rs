mod api;
mod auth;
mod backup;
mod cache;
mod config;
mod config_export;
mod discovery;
mod ethernet_ip;
mod expression;
mod flow_engine;
mod modbus;
mod models;
mod oee;
mod poller_manager;
mod retention;
mod s7;
mod security;
mod storage;
mod timeutil;
mod license;
mod notification;
mod opcua;
mod hub;
mod report;
mod report_scheduler;
mod analytics;
mod compliance;
mod compression;
mod deadband;
mod recipe;
mod video;

use api::AppState;
use base64::Engine;
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

    let args: Vec<String> = std::env::args().collect();

    const HELP_TEXT: &str = "\
voltrus-lite — Industrial Modbus monitoring dashboard

USAGE:
    voltrus-lite [CONFIG_PATH]              Start server (default: config.yaml)
    voltrus-lite --help                     Show this help
    voltrus-lite --license-status [CONFIG]  Check current license status
    voltrus-lite --generate-request <KEY>   Generate license-request.bin
    voltrus-lite --import-activation <FILE> [CONFIG]
                                            Import license-activation.bin
    voltrus-lite --online-activate <KEY> [CONFIG]
                                            Activate directly via internet
    voltrus-lite --reset-admin-password [CONFIG]
                                            Reset admin password (prints new password)
    voltrus-lite backup [OPTIONS]           Create backup archive
    voltrus-lite restore <BACKUP_FILE> [OPTIONS]
                                            Restore from backup archive
    voltrus-lite config export [DIR]        Export config to YAML files
    voltrus-lite config import [DIR]        Import config from YAML files
    voltrus-lite config validate [DIR]      Validate YAML config files
    voltrus-lite discover --range <CIDR>    Scan subnet for Modbus TCP devices

BACKUP OPTIONS:
    --output <PATH>    Output archive path (default: voltrus-backup-YYYYMMDD-HHMMSS.tar.gz)
    --config <PATH>    Config file path for db_path lookup (default: config.yaml)

RESTORE OPTIONS:
    <BACKUP_FILE>      Path to the .tar.gz backup archive
    --config <PATH>    Target config file path (default: config.yaml)
    --force            Skip confirmation prompt

DISCOVER OPTIONS:
    --range <CIDR>     Subnet to scan (e.g., 192.168.1.0/24)
    --port <PORT>      Modbus TCP port (default: 502)
    --config <PATH>    Config file path (default: config.yaml)

OPTIONS:
    --help                        Show this help message
    --license-status              Print license status as JSON (exit 1 if invalid)
    --generate-request <KEY>      Generate a license request file for offline activation
    --import-activation <FILE>    Import a signed activation file from the activation server
    --online-activate <KEY>       Connect to activation server and activate in one step
                                  (requires internet access)
    --reset-admin-password        Generate a new random password for admin user

EXAMPLES:
    voltrus-lite                              # Start with config.yaml
    voltrus-lite /etc/voltrus/prod.yaml       # Start with custom config
    voltrus-lite --online-activate VOLTRUS-XXXX   # Activate online
    voltrus-lite --reset-admin-password       # Reset lost admin password
    voltrus-lite --license-status             # Check license
    voltrus-lite backup                       # Create backup in current directory
    voltrus-lite backup --output /tmp/bak.tar.gz  # Custom output path
    voltrus-lite restore backup.tar.gz        # Restore from archive
    voltrus-lite restore backup.tar.gz --force  # Restore without confirmation
    voltrus-lite restore backup.tar.gz --config /etc/voltrus/prod.yaml
    voltrus-lite config export ./my-config    # Export config to directory
    voltrus-lite config import ./my-config    # Import config from directory
    voltrus-lite config validate ./my-config  # Validate config files
    voltrus-lite discover --range 192.168.1.0/24 --port 502
";

    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print!("{}", HELP_TEXT);
                std::process::exit(0);
            }
            "--generate-request" => {
                if args.len() < 3 {
                    eprintln!("Usage: {} --generate-request <LICENSE_KEY>", args[0]);
                    std::process::exit(1);
                }
                let path = license::request::generate_request(&args[2])?;
                println!("License request written to: {}", path);
                return Ok(());
            }
            "--import-activation" => {
                if args.len() < 3 {
                    eprintln!("Usage: {} --import-activation <PATH> [CONFIG_PATH]", args[0]);
                    std::process::exit(1);
                }
                let activation_path = &args[2];
                let config_path = args.get(3).map(|s| s.as_str()).unwrap_or("config.yaml");
                let app_config = config::load_config(config_path)?;
                let storage = Storage::new_with_key(
                    &app_config.db_path,
                    app_config.db_encryption_key.as_deref(),
                )?;
                let activation = license::activation::import_activation_file(activation_path)?;
                let fingerprint = license::fingerprint::collect_fingerprint();
                let row = crate::models::LicenseActivationRow {
                    license_key: activation.license_key.clone(),
                    fingerprint_hash: fingerprint.hash(),
                    activation_blob: std::fs::read(activation_path)?,
                    fingerprint_hostname: fingerprint.hostname,
                    fingerprint_cpu: fingerprint.cpu,
                    fingerprint_disk_serial: fingerprint.disk_serial,
                    fingerprint_mac: fingerprint.mac,
                    fingerprint_machine_id: fingerprint.machine_id,
                    activated_at: activation.issued_at,
                    expires_at: activation.expires_at,
                    tier: activation.tier,
                    device_limit: activation.device_limit,
                };
                storage.store_license_activation(&row).await?;
                println!("License activated successfully: {}", activation.license_key);
                return Ok(());
            }
            "--online-activate" => {
                if args.len() < 3 {
                    eprintln!("Usage: {} --online-activate <LICENSE_KEY> [CONFIG_PATH]", args[0]);
                    std::process::exit(1);
                }
                let license_key = &args[2];
                let config_path = args.get(3).map(|s| s.as_str()).unwrap_or("config.yaml");
                let app_config = config::load_config(config_path)?;
                let storage = Storage::new_with_key(
                    &app_config.db_path,
                    app_config.db_encryption_key.as_deref(),
                )?;

                eprintln!("Generating license request...");
                let request_data = {
                    let path = license::request::generate_request(license_key)?;
                    let data = std::fs::read(&path)?;
                    std::fs::remove_file(&path).ok();
                    data
                };

                eprintln!("Contacting activation server...");
                let client = reqwest::Client::new();
                let file_part = reqwest::multipart::Part::bytes(request_data.clone())
                    .file_name("license-request.bin")
                    .mime_str("application/octet-stream")?;
                let form = reqwest::multipart::Form::new()
                    .text("license_key", license_key.clone())
                    .part("file", file_part);

                let resp = client
                    .post("https://activate.voltrus.id/activate")
                    .multipart(form)
                    .timeout(std::time::Duration::from_secs(30))
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to connect to activation server: {}. Ensure you have internet access.", e))?;

                if !resp.status().is_success() {
                    anyhow::bail!("Activation server returned HTTP {}", resp.status());
                }

                let result: serde_json::Value = resp.json().await?;
                if !result["success"].as_bool().unwrap_or(false) {
                    let err = result["error"].as_str().unwrap_or("Unknown error");
                    anyhow::bail!("Activation failed: {}", err);
                }

                let activation_b64 = result["activation_file"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Activation server did not return activation file"))?;
                let activation_blob = base64::engine::general_purpose::STANDARD
                    .decode(activation_b64)?;

                let activation = license::activation::verify_activation_signature(&activation_blob)?;
                let fingerprint = license::fingerprint::collect_fingerprint();
                let row = crate::models::LicenseActivationRow {
                    license_key: activation.license_key.clone(),
                    fingerprint_hash: fingerprint.hash(),
                    activation_blob,
                    fingerprint_hostname: fingerprint.hostname,
                    fingerprint_cpu: fingerprint.cpu,
                    fingerprint_disk_serial: fingerprint.disk_serial,
                    fingerprint_mac: fingerprint.mac,
                    fingerprint_machine_id: fingerprint.machine_id,
                    activated_at: activation.issued_at,
                    expires_at: activation.expires_at,
                    tier: activation.tier,
                    device_limit: activation.device_limit,
                };
                storage.store_license_activation(&row).await?;
                println!("License activated successfully: {}", activation.license_key);
                return Ok(());
            }
            "--reset-admin-password" => {
                let config_path = args.get(2).map(|s| s.as_str()).unwrap_or("config.yaml");
                let app_config = config::load_config(config_path)?;
                let db_path = app_config.db_path.clone();
                let storage = Storage::new_with_key(
                    &db_path,
                    app_config.db_encryption_key.as_deref(),
                )?;
                match storage.get_user_by_username("admin").await? {
                    None => {
                        anyhow::bail!("Admin user not found. Start the server first to auto-seed the database.");
                    }
                    Some(user) => {
                        use rand::Rng;
                        let password: String = rand::rng()
                            .sample_iter(&rand::distr::Alphanumeric)
                            .take(16)
                            .map(char::from)
                            .collect();
                        let hash = crate::api::auth::hash_password(&password);
                        if let Err(e) = storage.update_user_password(user.id, &hash).await {
                            if e.to_string().contains("readonly") || e.to_string().contains("read-only") {
                                anyhow::bail!(
                                    "Database is read-only: {}\n\nFix: run with sudo, or change ownership:\n  sudo chown $USER:{}",
                                    db_path, db_path
                                );
                            }
                            return Err(e);
                        }
                        if let Err(e) = storage.set_must_change_password("admin", true).await {
                            if e.to_string().contains("readonly") || e.to_string().contains("read-only") {
                                anyhow::bail!(
                                    "Database is read-only: {}\n\nFix: run with sudo, or change ownership:\n  sudo chown $USER:{}",
                                    db_path, db_path
                                );
                            }
                            return Err(e);
                        }
                        println!("Admin password reset successfully.");
                        println!("New password: {}", password);
                        println!("You will be required to change this on next login.");
                    }
                }
                return Ok(());
            }
            "config" => {
                let subcommand = args.get(2).map(|s| s.as_str()).unwrap_or("");
                let path = args.get(3).map(|s| s.as_str()).unwrap_or("config");
                match subcommand {
                    "export" => {
                        let config_path = args.get(4).map(|s| s.as_str()).unwrap_or("config.yaml");
                        let app_config = config::load_config(config_path)?;
                        let storage = Storage::new_with_key(
                            &app_config.db_path,
                            app_config.db_encryption_key.as_deref(),
                        )?;
                        let status = license::verify_license(&storage).await;
                        if !status.is_valid() || !status.tier().backup_restore() {
                            eprintln!("Config export requires a valid Starter+ license (backup_restore feature).");
                            std::process::exit(1);
                        }
                        match config_export::export_config(&storage, path) {
                            Ok(_) => {
                                println!("Config exported to {path}/");
                                std::process::exit(0);
                            }
                            Err(e) => {
                                eprintln!("Export failed: {:#}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    "import" => {
                        let config_path = args.get(4).map(|s| s.as_str()).unwrap_or("config.yaml");
                        let app_config = config::load_config(config_path)?;
                        let storage = Storage::new_with_key(
                            &app_config.db_path,
                            app_config.db_encryption_key.as_deref(),
                        )?;
                        let status = license::verify_license(&storage).await;
                        if !status.is_valid() || !status.tier().backup_restore() {
                            eprintln!("Config import requires a valid Starter+ license (backup_restore feature).");
                            std::process::exit(1);
                        }
                        match config_export::import_config(&storage, path) {
                            Ok(_) => {
                                println!("Config imported from {path}/");
                                std::process::exit(0);
                            }
                            Err(e) => {
                                eprintln!("Import failed: {:#}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    "validate" => {
                        match config_export::validate_config(path) {
                            Ok(messages) => {
                                for msg in &messages {
                                    println!("{}", msg);
                                }
                                let has_errors = messages.iter().any(|m| m.starts_with("ERROR"));
                                std::process::exit(if has_errors { 1 } else { 0 });
                            }
                            Err(e) => {
                                eprintln!("Validation failed: {:#}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    _ => {
                        eprintln!("Usage: {} config [export|import|validate] [PATH]", args[0]);
                        eprintln!();
                        eprintln!("Commands:");
                        eprintln!("  export [DIR]   Export config to YAML files (default: config/)");
                        eprintln!("  import [DIR]   Import config from YAML files (default: config/)");
                        eprintln!("  validate [DIR] Validate YAML config files (default: config/)");
                        std::process::exit(1);
                    }
                }
            }
            "discover" => {
                let mut range = String::new();
                let mut port: u16 = 502;
                let mut config_path = "config.yaml".to_string();
                let mut i = 2;
                while i < args.len() {
                    match args[i].as_str() {
                        "--range" => {
                            if i + 1 >= args.len() {
                                eprintln!("Error: --range requires a CIDR argument (e.g., 192.168.1.0/24)");
                                std::process::exit(1);
                            }
                            range = args[i + 1].clone();
                            i += 2;
                        }
                        "--port" => {
                            if i + 1 >= args.len() {
                                eprintln!("Error: --port requires a port number");
                                std::process::exit(1);
                            }
                            port = args[i + 1].parse().unwrap_or(502);
                            i += 2;
                        }
                        "--config" => {
                            if i + 1 >= args.len() {
                                eprintln!("Error: --config requires a path argument");
                                std::process::exit(1);
                            }
                            config_path = args[i + 1].clone();
                            i += 2;
                        }
                        flag if flag.starts_with('-') => {
                            eprintln!("Unknown discover option: {}", flag);
                            eprintln!("Usage: {} discover --range <CIDR> [--port PORT]", args[0]);
                            std::process::exit(1);
                        }
                        _ => {
                            eprintln!("Unexpected argument: {}", args[i]);
                            eprintln!("Usage: {} discover --range <CIDR> [--port PORT]", args[0]);
                            std::process::exit(1);
                        }
                    }
                }
                if range.is_empty() {
                    eprintln!("Usage: {} discover --range <CIDR> [--port PORT]", args[0]);
                    std::process::exit(1);
                }
                let app_config = config::load_config(&config_path)?;
                let storage = Storage::new_with_key(
                    &app_config.db_path,
                    app_config.db_encryption_key.as_deref(),
                )?;
                let status = license::verify_license(&storage).await;
                if !status.is_valid() || !status.tier().auto_discovery() {
                    eprintln!("Network discovery requires a valid Starter+ license (auto_discovery feature).");
                    std::process::exit(1);
                }
                eprintln!("Scanning {} port {} for Modbus TCP devices...", range, port);
                let devices = discovery::scan_network(&range, port, 500).await?;
                let json = serde_json::to_string_pretty(&devices)?;
                println!("{}", json);
                if devices.is_empty() {
                    eprintln!("No Modbus devices found on {} port {}", range, port);
                } else {
                    eprintln!("Found {} device(s)", devices.len());
                }
                return Ok(());
            }
            "backup" => {
                let mut config_path = "config.yaml".to_string();
                let mut output_path: Option<String> = None;
                let mut i = 2;
                while i < args.len() {
                    match args[i].as_str() {
                        "--config" => {
                            if i + 1 >= args.len() {
                                eprintln!("Error: --config requires a path argument");
                                std::process::exit(1);
                            }
                            config_path = args[i + 1].clone();
                            i += 2;
                        }
                        "--output" => {
                            if i + 1 >= args.len() {
                                eprintln!("Error: --output requires a path argument");
                                std::process::exit(1);
                            }
                            output_path = Some(args[i + 1].clone());
                            i += 2;
                        }
                        flag if flag.starts_with('-') => {
                            eprintln!("Unknown backup option: {}", flag);
                            eprintln!("Usage: {} backup [--config <PATH>] [--output <PATH>]", args[0]);
                            std::process::exit(1);
                        }
                        _ => {
                            eprintln!("Unexpected argument: {}", args[i]);
                            eprintln!("Usage: {} backup [--config <PATH>] [--output <PATH>]", args[0]);
                            std::process::exit(1);
                        }
                    }
                }
                let app_config = config::load_config(&config_path)?;
                let opts = backup::BackupOptions {
                    config_path,
                    db_path: app_config.db_path.clone(),
                    output_path,
                };
                match backup::run_backup(opts) {
                    Ok(_) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("Backup failed: {:#}", e);
                        std::process::exit(1);
                    }
                }
            }
            "restore" => {
                let mut config_path = "config.yaml".to_string();
                let mut archive_path: Option<String> = None;
                let mut force = false;
                let mut i = 2;
                while i < args.len() {
                    match args[i].as_str() {
                        "--config" => {
                            if i + 1 >= args.len() {
                                eprintln!("Error: --config requires a path argument");
                                std::process::exit(1);
                            }
                            config_path = args[i + 1].clone();
                            i += 2;
                        }
                        "--force" => {
                            force = true;
                            i += 1;
                        }
                        flag if flag.starts_with('-') => {
                            eprintln!("Unknown restore option: {}", flag);
                            eprintln!("Usage: {} restore <BACKUP_FILE> [--config <PATH>] [--force]", args[0]);
                            std::process::exit(1);
                        }
                        arg => {
                            if archive_path.is_some() {
                                eprintln!("Unexpected argument: {}", arg);
                                eprintln!("Usage: {} restore <BACKUP_FILE> [--config <PATH>] [--force]", args[0]);
                                std::process::exit(1);
                            }
                            archive_path = Some(arg.to_string());
                            i += 1;
                        }
                    }
                }
                let archive_path = match archive_path {
                    Some(p) => p,
                    None => {
                        eprintln!("Usage: {} restore <BACKUP_FILE> [--config <PATH>] [--force]", args[0]);
                        std::process::exit(1);
                    }
                };
                let app_config = config::load_config(&config_path)?;
                let opts = backup::RestoreOptions {
                    archive_path,
                    config_path,
                    db_path: app_config.db_path.clone(),
                    force,
                };
                match backup::run_restore(opts) {
                    Ok(_) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("Restore failed: {:#}", e);
                        std::process::exit(1);
                    }
                }
            }
            "--license-status" => {
                let config_path = args.get(2).map(|s| s.as_str()).unwrap_or("config.yaml");
                let app_config = config::load_config(config_path)?;
                let storage = Storage::new_with_key(
                    &app_config.db_path,
                    app_config.db_encryption_key.as_deref(),
                )?;
                let status = license::verify_license(&storage).await;
                println!("{}", serde_json::to_string_pretty(&status)?);
                std::process::exit(if status.is_valid() { 0 } else { 1 });
            }
            other if other.starts_with('-') => {
                eprintln!("Unknown option: {}", other);
                eprintln!("Run '{} --help' for usage.", args[0]);
                std::process::exit(1);
            }
            _ => {}
        }
    }

    let config_path = args.get(1).map(|s| s.as_str()).unwrap_or("config.yaml");
    let app_config = config::load_config(config_path)?;

    tracing::info!(
        "Loaded config: {} devices in YAML, port {}",
        app_config.devices.len(),
        app_config.server_port
    );

    let cache = DeviceCache::new();
    let storage = Storage::new_with_key(
        &app_config.db_path,
        app_config.db_encryption_key.as_deref(),
    )?;
    let license_status = license::resolve_license_status(&storage).await;
    let license_tier = license_status.tier();
    let feature_flags = license_status.feature_flags();

    if !license_status.is_valid() {
        tracing::warn!("License status: {:?} — skipping Modbus polling", license_status);
    }

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

    // Load device configs from DB and start pollers (only with valid license)
    let poller_manager = PollerManager::new(cache.clone());
    if license_status.is_valid() {
        let device_configs = storage.get_all_device_configs().await?;
        let device_limit = license_tier.max_devices();
        let mut started_count: u32 = 0;
        for config in &device_configs {
            if let Some(limit) = device_limit {
                if started_count >= limit {
                    tracing::warn!(
                        "Device '{}' skipped — {} tier limit of {} devices reached. Upgrade at https://voltrus.id",
                        config.id,
                        serde_json::to_string(&license_tier).unwrap_or_else(|_| "unknown".to_string()),
                        limit
                    );
                    continue;
                }
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
        } else if let Ok(configs) = storage.get_all_opcua_device_configs().await {
            if !configs.is_empty() {
                tracing::warn!(
                    "OPC-UA devices configured but Professional tier required. Upgrade at https://voltrus.id"
                );
            }
        }
    }

    // Start hub site connectors (Professional tier only)
    let hub_manager = {
        if license_tier.multi_site() {
            let hm = crate::api::hub::HubManager::new();
            match storage.get_enabled_hub_sites().await {
                Ok(sites) => {
                    for site in &sites {
                        hm.start_site(site.clone(), cache.clone(), storage.clone()).await;
                        tracing::info!("Started hub connector for site '{}' (id={})", site.name, site.id);
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
                tracing::info!("Email notifier initialized (from: {})", smtp_cfg.from_address);
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
            tracing::info!("PDF reporting enabled but SMTP not configured — report scheduler not started");
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
                            // Non-repeating: check for any unresolved alarm (not just un-acked)
                            let has_unresolved = storage.has_unresolved_alarm(&source.key, "critical").await.unwrap_or(false);
                            let in_cooldown = storage.is_in_cooldown(&source.key, "critical").await.unwrap_or(false);
                            if !has_unresolved && !in_cooldown {
                                let msg = format!(
                                    "{} reached {}{} (critical: {}{})",
                                    source.name, value, source.unit, source.crit_val, source.unit
                                );
                                match storage.create_alarm(&source.key, "critical", value, source.crit_val, &msg).await {
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
                                            message: msg,
                                            triggered_at: crate::timeutil::now_secs(),
                                            resolved_at: None,
                                            acknowledged_at: None,
                                            acknowledged_by: None,
                                            cooldown_until: None,
                                            shelved_until: None,
                                            shelved_by: None,
                                            shelved_reason: None,
                                            deadband: 0.0,
                                            state: "active".to_string(),
                                        };
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
                            let has_unresolved = storage.has_unresolved_alarm(&source.key, "warning").await.unwrap_or(false);
                            let in_cooldown = storage.is_in_cooldown(&source.key, "warning").await.unwrap_or(false);
                            if !has_unresolved && !in_cooldown {
                                let msg = format!(
                                    "{} reached {}{} (warning: {}{})",
                                    source.name, value, source.unit, source.warn_val, source.unit
                                );
                                match storage.create_alarm(&source.key, "warning", value, source.warn_val, &msg).await {
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
                                            message: msg,
                                            triggered_at: crate::timeutil::now_secs(),
                                            resolved_at: None,
                                            acknowledged_at: None,
                                            acknowledged_by: None,
                                            cooldown_until: None,
                                            shelved_until: None,
                                            shelved_by: None,
                                            shelved_reason: None,
                                            deadband: 0.0,
                                            state: "active".to_string(),
                                        };
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
                            let _ = storage.resolve_active_alarms(&source.key, Some(value)).await;
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

    // Spawn OEE monitor (Starter tier and above)
    if license_tier.oee() {
        let oee_cache = cache.clone();
        let oee_storage = storage.clone();
        tokio::spawn(async move {
            oee::run_oee_monitor(oee_cache, oee_storage).await;
        });
        tracing::info!("OEE monitor started");
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
    let intrusion_detector = crate::security::IntrusionDetector::new();
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
