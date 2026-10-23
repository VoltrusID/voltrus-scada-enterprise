use axum::{Json, extract::State};
use serde_json::json;

use crate::api::AppState;
use crate::license::LicenseStatus;

/// POST /api/v1/engineering/polling/start
///
/// Starts polling for Engineering mode. Only works when the current license
/// status is Engineering or EngineeringPollingStopped (with remaining budget).
pub async fn start_polling(State(state): State<AppState>) -> Json<serde_json::Value> {
    // Snapshot the current license status (clone to drop the lock before any await)
    let snapshot = {
        let ls = state.license_status.read().unwrap();
        ls.clone()
    };

    match &snapshot {
        LicenseStatus::Engineering {
            polling_active: true,
            ..
        } => Json(json!({"ok": true, "message": "Polling already active"})),
        LicenseStatus::Engineering {
            started_at,
            tier,
            feature_flags,
            daily_budget_secs,
            daily_used_secs,
            ..
        } => {
            let budget = *daily_budget_secs;
            let used = *daily_used_secs;
            let remaining = (budget - used).max(0);

            if remaining <= 0 {
                {
                    let mut ls = state.license_status.write().unwrap();
                    *ls = LicenseStatus::EngineeringPollingStopped {
                        started_at: *started_at,
                        daily_budget_secs: budget,
                        daily_used_secs: used,
                        message:
                            "Daily polling budget exhausted. Resumes tomorrow at midnight UTC."
                                .to_string(),
                    };
                }
                state.poller_manager.stop_all().await;
                return Json(json!({"ok": false, "error": "Daily polling budget exhausted"}));
            }

            // Activate polling
            {
                let mut ls = state.license_status.write().unwrap();
                *ls = LicenseStatus::Engineering {
                    started_at: *started_at,
                    polling_elapsed_secs: used,
                    polling_remaining_secs: remaining,
                    daily_budget_secs: budget,
                    daily_used_secs: used,
                    polling_active: true,
                    tier: tier.clone(),
                    feature_flags: feature_flags.clone(),
                };
            }

            // Restart pollers
            let device_configs = match state.storage.get_all_device_configs().await {
                Ok(configs) => configs,
                Err(e) => {
                    tracing::error!("Failed to load device configs for polling start: {}", e);
                    return Json(json!({"ok": false, "error": "Failed to load device configs"}));
                }
            };

            let device_limit = state.license_tier.max_devices();
            let mut started_count: u32 = 0;
            for config in &device_configs {
                if let Some(limit) = device_limit {
                    if started_count >= limit {
                        break;
                    }
                }
                state.poller_manager.start(config.clone()).await;
                started_count += 1;
            }
            tracing::info!("Engineering polling started — {} device(s)", started_count);

            Json(
                json!({"ok": true, "message": "Polling started", "devices_started": started_count}),
            )
        }
        LicenseStatus::EngineeringPollingStopped {
            started_at,
            daily_budget_secs,
            daily_used_secs,
            ..
        } => {
            let budget = *daily_budget_secs;
            let used = *daily_used_secs;
            let remaining = (budget - used).max(0);

            if remaining <= 0 {
                return Json(
                    json!({"ok": false, "error": "Daily polling budget exhausted. Resumes tomorrow at midnight UTC."}),
                );
            }

            let tier = crate::license::LicenseTier::Engineering;
            let feature_flags = tier.to_feature_flags();
            {
                let mut ls = state.license_status.write().unwrap();
                *ls = LicenseStatus::Engineering {
                    started_at: *started_at,
                    polling_elapsed_secs: used,
                    polling_remaining_secs: remaining,
                    daily_budget_secs: budget,
                    daily_used_secs: used,
                    polling_active: true,
                    tier: tier.clone(),
                    feature_flags,
                };
            }

            let device_configs = match state.storage.get_all_device_configs().await {
                Ok(configs) => configs,
                Err(e) => {
                    tracing::error!("Failed to load device configs for polling start: {}", e);
                    return Json(json!({"ok": false, "error": "Failed to load device configs"}));
                }
            };

            let mut started_count: u32 = 0;
            let device_limit = state.license_tier.max_devices();
            for config in &device_configs {
                if let Some(limit) = device_limit {
                    if started_count >= limit {
                        break;
                    }
                }
                state.poller_manager.start(config.clone()).await;
                started_count += 1;
            }
            tracing::info!(
                "Engineering polling restarted — {} device(s)",
                started_count
            );

            Json(
                json!({"ok": true, "message": "Polling restarted", "devices_started": started_count}),
            )
        }
        _ => Json(json!({"ok": false, "error": "Not in Engineering mode"})),
    }
}

/// POST /api/v1/engineering/polling/stop
///
/// Stops polling for Engineering mode. Only works when the current license
/// status is Engineering with polling active.
pub async fn stop_polling(State(state): State<AppState>) -> Json<serde_json::Value> {
    // Snapshot the current license status (clone to drop the lock before any await)
    let snapshot = {
        let ls = state.license_status.read().unwrap();
        ls.clone()
    };

    match &snapshot {
        LicenseStatus::Engineering {
            polling_active: false,
            ..
        } => Json(json!({"ok": true, "message": "Polling already paused"})),
        LicenseStatus::Engineering {
            started_at,
            polling_elapsed_secs,
            polling_remaining_secs,
            daily_budget_secs,
            daily_used_secs,
            tier,
            feature_flags,
            ..
        } => {
            // Pause polling (mark as inactive but keep Engineering status)
            {
                let mut ls = state.license_status.write().unwrap();
                *ls = LicenseStatus::Engineering {
                    started_at: *started_at,
                    polling_elapsed_secs: *polling_elapsed_secs,
                    polling_remaining_secs: *polling_remaining_secs,
                    daily_budget_secs: *daily_budget_secs,
                    daily_used_secs: *daily_used_secs,
                    polling_active: false,
                    tier: tier.clone(),
                    feature_flags: feature_flags.clone(),
                };
            }

            // Stop all pollers
            state.poller_manager.stop_all().await;
            tracing::info!("Engineering polling paused by user");

            Json(json!({"ok": true, "message": "Polling paused"}))
        }
        _ => Json(json!({"ok": false, "error": "Not in Engineering mode"})),
    }
}
