use base64::Engine;

use crate::config;
use crate::storage::Storage;

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

/// Handle CLI subcommands. Returns true if a subcommand was handled (should exit).
pub async fn handle_cli(args: &[String]) -> bool {
    if args.len() <= 1 {
        return false;
    }

    match args[1].as_str() {
        "--help" | "-h" => {
            print!("{}", HELP_TEXT);
            std::process::exit(0);
        }
        "--generate-request" => handle_generate_request(args),
        "--import-activation" => handle_import_activation(args).await,
        "--online-activate" => handle_online_activate(args).await,
        "--reset-admin-password" => handle_reset_admin_password(args).await,
        "config" => handle_config(args),
        "discover" => handle_discover(args).await,
        "backup" => handle_backup(args),
        "restore" => handle_restore(args),
        "--license-status" => handle_license_status(args).await,
        other if other.starts_with('-') => {
            eprintln!("Unknown option: {}", other);
            eprintln!("Run '{} --help' for usage.", args[0]);
            std::process::exit(1);
        }
        _ => false,
    }
}

fn handle_generate_request(args: &[String]) -> bool {
    if args.len() < 3 {
        eprintln!("Usage: {} --generate-request <LICENSE_KEY>", args[0]);
        std::process::exit(1);
    }
    match crate::license::request::generate_request(&args[2]) {
        Ok(path) => {
            println!("License request written to: {}", path);
            true
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

async fn handle_import_activation(args: &[String]) -> bool {
    if args.len() < 3 {
        eprintln!("Usage: {} --import-activation <PATH> [CONFIG_PATH]", args[0]);
        std::process::exit(1);
    }
    let activation_path = &args[2];
    let config_path = args.get(3).map(|s| s.as_str()).unwrap_or("config.yaml");
    let app_config = config::load_config(config_path).unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });
    let storage = Storage::new_with_key(&app_config.db_path, app_config.db_encryption_key.as_deref())
        .unwrap_or_else(|e| {
            eprintln!("DB error: {e}");
            std::process::exit(1);
        });
    let activation = crate::license::activation::import_activation_file(activation_path).unwrap_or_else(|e| {
        eprintln!("Activation error: {e}");
        std::process::exit(1);
    });
    let fingerprint = crate::license::fingerprint::collect_fingerprint();
    let row = crate::models::LicenseActivationRow {
        license_key: activation.license_key.clone(),
        fingerprint_hash: fingerprint.hash(),
        activation_blob: std::fs::read(activation_path).unwrap_or_else(|e| {
            eprintln!("Read error: {e}");
            std::process::exit(1);
        }),
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
    match storage.store_license_activation(&row).await {
        Ok(()) => {
            println!("License activated successfully: {}", activation.license_key);
            true
        }
        Err(e) => {
            eprintln!("Store error: {e}");
            std::process::exit(1);
        }
    }
}

async fn handle_online_activate(args: &[String]) -> bool {
    if args.len() < 3 {
        eprintln!("Usage: {} --online-activate <LICENSE_KEY> [CONFIG_PATH]", args[0]);
        std::process::exit(1);
    }
    let license_key = &args[2];
    let config_path = args.get(3).map(|s| s.as_str()).unwrap_or("config.yaml");
    let app_config = config::load_config(config_path).unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });
    let storage = Storage::new_with_key(&app_config.db_path, app_config.db_encryption_key.as_deref())
        .unwrap_or_else(|e| {
            eprintln!("DB error: {e}");
            std::process::exit(1);
        });

    eprintln!("Generating license request...");
    let request_data = {
        let path = crate::license::request::generate_request(license_key).unwrap_or_else(|e| {
            eprintln!("Request error: {e}");
            std::process::exit(1);
        });
        let data = std::fs::read(&path).unwrap_or_else(|e| {
            eprintln!("Read error: {e}");
            std::process::exit(1);
        });
        std::fs::remove_file(&path).ok();
        data
    };

    eprintln!("Contacting activation server...");
    let client = reqwest::Client::new();
    let file_part = reqwest::multipart::Part::bytes(request_data.clone())
        .file_name("license-request.bin")
        .mime_str("application/octet-stream").unwrap_or_else(|e| {
            eprintln!("MIME error: {e}");
            std::process::exit(1);
        });
    let form = reqwest::multipart::Form::new()
        .text("license_key", license_key.clone())
        .part("file", file_part);

    let resp = client
        .post("https://activate.voltrus.id/activate")
        .multipart(form)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to connect to activation server: {}. Ensure you have internet access.", e);
            std::process::exit(1);
        });

    if !resp.status().is_success() {
        eprintln!("Activation server returned HTTP {}", resp.status());
        std::process::exit(1);
    }

    let result: serde_json::Value = resp.json().await.unwrap_or_else(|e| {
        eprintln!("Parse error: {e}");
        std::process::exit(1);
    });
    if !result["success"].as_bool().unwrap_or(false) {
        let err = result["error"].as_str().unwrap_or("Unknown error");
        eprintln!("Activation failed: {}", err);
        std::process::exit(1);
    }

    let activation_b64 = result["activation_file"]
        .as_str()
        .unwrap_or_else(|| {
            eprintln!("Activation server did not return activation file");
            std::process::exit(1);
        });
    let activation_blob = base64::engine::general_purpose::STANDARD
        .decode(activation_b64).unwrap_or_else(|e| {
            eprintln!("Decode error: {e}");
            std::process::exit(1);
        });

    let activation = crate::license::activation::verify_activation_signature(&activation_blob).unwrap_or_else(|e| {
        eprintln!("Signature error: {e}");
        std::process::exit(1);
    });
    let fingerprint = crate::license::fingerprint::collect_fingerprint();
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
    match storage.store_license_activation(&row).await {
        Ok(()) => {
            println!("License activated successfully: {}", activation.license_key);
            true
        }
        Err(e) => {
            eprintln!("Store error: {e}");
            std::process::exit(1);
        }
    }
}

async fn handle_reset_admin_password(args: &[String]) -> bool {
    let config_path = args.get(2).map(|s| s.as_str()).unwrap_or("config.yaml");
    let app_config = config::load_config(config_path).unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });
    let db_path = app_config.db_path.clone();
    let storage = Storage::new_with_key(&db_path, app_config.db_encryption_key.as_deref())
        .unwrap_or_else(|e| {
            eprintln!("DB error: {e}");
            std::process::exit(1);
        });
    match storage.get_user_by_username("admin").await {
        Ok(None) => {
            eprintln!("Admin user not found. Start the server first to auto-seed the database.");
            std::process::exit(1);
        }
        Ok(Some(user)) => {
            use rand::Rng;
            let password: String = rand::rng()
                .sample_iter(&rand::distr::Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();
            let hash = crate::api::auth::hash_password(&password);
            if let Err(e) = storage.update_user_password(user.id, &hash).await {
                if e.to_string().contains("readonly") || e.to_string().contains("read-only") {
                    eprintln!(
                        "Database is read-only: {}\n\nFix: run with sudo, or change ownership:\n  sudo chown $USER:{}",
                        db_path, db_path
                    );
                    std::process::exit(1);
                }
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
            if let Err(e) = storage.set_must_change_password("admin", true).await {
                if e.to_string().contains("readonly") || e.to_string().contains("read-only") {
                    eprintln!(
                        "Database is read-only: {}\n\nFix: run with sudo, or change ownership:\n  sudo chown $USER:{}",
                        db_path, db_path
                    );
                    std::process::exit(1);
                }
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
            println!("Admin password reset successfully.");
            println!("New password: {}", password);
            println!("You will be required to change this on next login.");
            true
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

fn handle_config(args: &[String]) -> bool {
    let subcommand = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let path = args.get(3).map(|s| s.as_str()).unwrap_or("config");
    match subcommand {
        "export" => {
            let config_path = args.get(4).map(|s| s.as_str()).unwrap_or("config.yaml");
            let app_config = config::load_config(config_path).unwrap_or_else(|e| {
                eprintln!("Config error: {e}");
                std::process::exit(1);
            });
            let storage = Storage::new_with_key(&app_config.db_path, app_config.db_encryption_key.as_deref())
                .unwrap_or_else(|e| {
                    eprintln!("DB error: {e}");
                    std::process::exit(1);
                });
            // Check license synchronously (blocking context in non-async fn)
            let rt = tokio::runtime::Runtime::new().unwrap();
            let status = rt.block_on(crate::license::verify_license(&storage));
            if !status.is_valid() || !status.tier().backup_restore() {
                eprintln!("Config export requires a valid Starter+ license (backup_restore feature).");
                std::process::exit(1);
            }
            match crate::config_export::export_config(&storage, path) {
                Ok(_) => {
                    println!("Config exported to {path}/");
                    true
                }
                Err(e) => {
                    eprintln!("Export failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        "import" => {
            let config_path = args.get(4).map(|s| s.as_str()).unwrap_or("config.yaml");
            let app_config = config::load_config(config_path).unwrap_or_else(|e| {
                eprintln!("Config error: {e}");
                std::process::exit(1);
            });
            let storage = Storage::new_with_key(&app_config.db_path, app_config.db_encryption_key.as_deref())
                .unwrap_or_else(|e| {
                    eprintln!("DB error: {e}");
                    std::process::exit(1);
                });
            let rt = tokio::runtime::Runtime::new().unwrap();
            let status = rt.block_on(crate::license::verify_license(&storage));
            if !status.is_valid() || !status.tier().backup_restore() {
                eprintln!("Config import requires a valid Starter+ license (backup_restore feature).");
                std::process::exit(1);
            }
            match crate::config_export::import_config(&storage, path) {
                Ok(_) => {
                    println!("Config imported from {path}/");
                    true
                }
                Err(e) => {
                    eprintln!("Import failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        "validate" => {
            match crate::config_export::validate_config(path) {
                Ok(messages) => {
                    for msg in &messages {
                        println!("{}", msg);
                    }
                    let has_errors = messages.iter().any(|m| m.starts_with("ERROR"));
                    std::process::exit(if has_errors { 1 } else { 0 });
                }
                Err(e) => {
                    eprintln!("Validation failed: {e}");
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

async fn handle_discover(args: &[String]) -> bool {
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
    let app_config = config::load_config(&config_path).unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });
    let storage = Storage::new_with_key(&app_config.db_path, app_config.db_encryption_key.as_deref())
        .unwrap_or_else(|e| {
            eprintln!("DB error: {e}");
            std::process::exit(1);
        });
    let status = crate::license::verify_license(&storage).await;
    if !status.is_valid() || !status.tier().auto_discovery() {
        eprintln!("Network discovery requires a valid Starter+ license (auto_discovery feature).");
        std::process::exit(1);
    }
    eprintln!("Scanning {} port {} for Modbus TCP devices...", range, port);
    match crate::discovery::scan_network(&range, port, 500).await {
        Ok(devices) => {
            let json = serde_json::to_string_pretty(&devices).unwrap_or_default();
            println!("{}", json);
            if devices.is_empty() {
                eprintln!("No Modbus devices found on {} port {}", range, port);
            } else {
                eprintln!("Found {} device(s)", devices.len());
            }
            true
        }
        Err(e) => {
            eprintln!("Scan error: {e}");
            std::process::exit(1);
        }
    }
}

fn handle_backup(args: &[String]) -> bool {
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
    let app_config = config::load_config(&config_path).unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });
    let opts = crate::backup::BackupOptions {
        config_path,
        db_path: app_config.db_path.clone(),
        output_path,
    };
    match crate::backup::run_backup(opts) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Backup failed: {e}");
            std::process::exit(1);
        }
    }
}

fn handle_restore(args: &[String]) -> bool {
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
    let app_config = config::load_config(&config_path).unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });
    let opts = crate::backup::RestoreOptions {
        archive_path,
        config_path,
        db_path: app_config.db_path.clone(),
        force,
    };
    match crate::backup::run_restore(opts) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Restore failed: {e}");
            std::process::exit(1);
        }
    }
}

async fn handle_license_status(args: &[String]) -> bool {
    let config_path = args.get(2).map(|s| s.as_str()).unwrap_or("config.yaml");
    let app_config = config::load_config(config_path).unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });
    let storage = Storage::new_with_key(&app_config.db_path, app_config.db_encryption_key.as_deref())
        .unwrap_or_else(|e| {
            eprintln!("DB error: {e}");
            std::process::exit(1);
        });
    let status = crate::license::verify_license(&storage).await;
    match serde_json::to_string_pretty(&status) {
        Ok(json) => {
            println!("{}", json);
            std::process::exit(if status.is_valid() { 0 } else { 1 });
        }
        Err(e) => {
            eprintln!("Serialize error: {e}");
            std::process::exit(1);
        }
    }
}
