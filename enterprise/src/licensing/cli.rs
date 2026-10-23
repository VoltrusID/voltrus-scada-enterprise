use base64::Engine;
use voltrus::license::LicenseStatus;
use voltrus::storage::Storage;

use super::{online, request, verification};

pub const HELP: &str = "\
LICENCE:
    voltrus --license-status [CONFIG]  Print licence status as JSON
    voltrus --generate-request <KEY>   Write license-request.bin for offline activation
    voltrus --import-activation <FILE> [CONFIG]
                                       Import a signed license-activation.bin
    voltrus --online-activate <KEY> [CONFIG]
                                       Activate directly via the internet
";

/// Handle the licence subcommands. Returns true when one was handled.
pub async fn handle(args: &[String]) -> bool {
    match args.get(1).map(String::as_str) {
        Some("--generate-request") => generate_request(args),
        Some("--import-activation") => import_activation(args).await,
        Some("--online-activate") => online_activate(args).await,
        Some("--license-status") => license_status(args).await,
        _ => false,
    }
}

fn open_storage(config_path: &str) -> Storage {
    let app_config = voltrus::config::load_config(config_path).unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });
    Storage::new_with_key(&app_config.db_path, app_config.db_encryption_key.as_deref())
        .unwrap_or_else(|e| {
            eprintln!("DB error: {e}");
            std::process::exit(1);
        })
}

fn generate_request(args: &[String]) -> bool {
    if args.len() < 3 {
        eprintln!("Usage: {} --generate-request <LICENSE_KEY>", args[0]);
        std::process::exit(1);
    }
    match request::generate_request(&args[2]) {
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

async fn import_activation(args: &[String]) -> bool {
    if args.len() < 3 {
        eprintln!(
            "Usage: {} --import-activation <PATH> [CONFIG_PATH]",
            args[0]
        );
        std::process::exit(1);
    }
    let storage = open_storage(args.get(3).map(String::as_str).unwrap_or("config.yaml"));
    let blob = std::fs::read(&args[2]).unwrap_or_else(|e| {
        eprintln!("Read error: {e}");
        std::process::exit(1);
    });
    store(&storage, blob).await
}

async fn online_activate(args: &[String]) -> bool {
    if args.len() < 3 {
        eprintln!(
            "Usage: {} --online-activate <LICENSE_KEY> [CONFIG_PATH]",
            args[0]
        );
        std::process::exit(1);
    }
    let storage = open_storage(args.get(3).map(String::as_str).unwrap_or("config.yaml"));
    eprintln!("Contacting activation server...");
    let blob = online::fetch_activation(&args[2])
        .await
        .unwrap_or_else(|e| {
            eprintln!("{e:#}");
            std::process::exit(1);
        });
    store(&storage, blob).await
}

async fn store(storage: &Storage, blob: Vec<u8>) -> bool {
    match online::store_activation(storage, blob).await {
        Ok(activation) => {
            println!("License activated successfully: {}", activation.license_key);
            true
        }
        Err(e) => {
            eprintln!("Activation error: {e:#}");
            std::process::exit(1);
        }
    }
}

async fn license_status(args: &[String]) -> bool {
    let storage = open_storage(args.get(2).map(String::as_str).unwrap_or("config.yaml"));
    let status = verification::resolve_license_status(&storage).await;
    match serde_json::to_string_pretty(&status) {
        Ok(json) => {
            println!("{}", json);
            std::process::exit(if matches!(status, LicenseStatus::Valid { .. }) {
                0
            } else {
                1
            });
        }
        Err(e) => {
            eprintln!("Serialize error: {e}");
            std::process::exit(1);
        }
    }
}

/// Decode a base64 activation file from the HTTP API.
pub fn decode_activation_b64(b64: &str) -> anyhow::Result<Vec<u8>> {
    Ok(base64::engine::general_purpose::STANDARD.decode(b64.trim())?)
}
