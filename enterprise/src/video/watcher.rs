//! Snapshots on alarm, and snapshot retention.
//!
//! Every [`POLL_SECS`] the active node looks for alarms raised since the last
//! look and takes one snapshot from every enabled camera that has
//! `snapshot_on_alarm`, at most one per camera per [`CAMERA_COOLDOWN_SECS`].
//! Once an hour it deletes snapshots older than each camera's `retention_hours`.

use std::collections::HashMap;
use std::sync::Arc;

use voltrus::models::{Alarm, Camera};
use voltrus::storage::Storage;

use super::StreamProxy;

pub const POLL_SECS: u64 = 2;
pub const CAMERA_COOLDOWN_SECS: i64 = 30;
const RETENTION_EVERY_SECS: i64 = 3600;

/// Alarms raised after `since`, oldest first. Ties at `since` are new when
/// their id is above `last_id`.
pub fn new_alarms(alarms: &[Alarm], since: i64, last_id: i64) -> Vec<&Alarm> {
    let mut out: Vec<&Alarm> = alarms
        .iter()
        .filter(|a| a.triggered_at > since || (a.triggered_at == since && a.id > last_id))
        .collect();
    out.sort_by_key(|a| (a.triggered_at, a.id));
    out
}

/// Cameras that take a snapshot for an alarm at `now`.
pub fn cameras_to_capture<'a>(
    cameras: &'a [Camera],
    last_capture: &HashMap<i64, i64>,
    now: i64,
) -> Vec<&'a Camera> {
    cameras
        .iter()
        .filter(|c| c.enabled && c.snapshot_on_alarm)
        .filter(|c| {
            last_capture
                .get(&c.id)
                .is_none_or(|t| now - t >= CAMERA_COOLDOWN_SECS)
        })
        .collect()
}

pub fn start(storage: Storage, proxy: Arc<StreamProxy>) {
    tokio::spawn(async move {
        let mut since = voltrus::timeutil::now_secs();
        let mut last_id = 0i64;
        let mut last_capture: HashMap<i64, i64> = HashMap::new();
        let mut last_retention = 0i64;
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(POLL_SECS));
        loop {
            ticker.tick().await;
            if !voltrus::node_role::is_active() {
                // A node that takes over starts from its own present.
                since = voltrus::timeutil::now_secs();
                continue;
            }
            let now = voltrus::timeutil::now_secs();
            if now - last_retention >= RETENTION_EVERY_SECS {
                last_retention = now;
                if let Ok(cameras) = storage.list_cameras().await {
                    for c in &cameras {
                        match storage
                            .cleanup_expired_snapshots(c.id, c.retention_hours)
                            .await
                        {
                            Ok(n) if n > 0 => tracing::info!(
                                "camera {}: removed {n} snapshot(s) older than {} h",
                                c.name,
                                c.retention_hours
                            ),
                            Ok(_) => {}
                            Err(e) => tracing::warn!("camera {}: retention: {e:#}", c.name),
                        }
                    }
                }
            }

            let Ok(alarms) = storage.list_alarms(None, 50).await else {
                continue;
            };
            let fresh: Vec<Alarm> = new_alarms(&alarms, since, last_id)
                .into_iter()
                .cloned()
                .collect();
            let Some(newest) = fresh.last() else {
                continue;
            };
            since = newest.triggered_at;
            last_id = newest.id;
            let Ok(cameras) = storage.list_cameras().await else {
                continue;
            };
            let targets: Vec<Camera> = cameras_to_capture(&cameras, &last_capture, now)
                .into_iter()
                .cloned()
                .collect();
            if targets.is_empty() {
                continue;
            }
            // One snapshot per camera for this burst, labelled with its first alarm.
            let alarm = &fresh[0];
            let label = format!(
                "alarm {}: {} ({})",
                alarm.id, alarm.source_key, alarm.message
            );
            for camera in targets {
                last_capture.insert(camera.id, now);
                let (storage, proxy, label) = (storage.clone(), proxy.clone(), label.clone());
                tokio::spawn(async move {
                    match proxy.capture_snapshot(&camera, Some(&label)).await {
                        Ok(s) => {
                            if let Err(e) = storage
                                .create_camera_snapshot(
                                    s.camera_id,
                                    s.timestamp,
                                    s.alarm_source.as_deref(),
                                    &s.file_path,
                                    s.file_size,
                                )
                                .await
                            {
                                tracing::warn!("camera {}: saving snapshot: {e:#}", camera.name);
                            } else {
                                tracing::info!("camera {}: snapshot for {label}", camera.name);
                            }
                        }
                        Err(e) => tracing::warn!("camera {}: alarm snapshot: {e:#}", camera.name),
                    }
                });
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alarm(id: i64, at: i64) -> Alarm {
        serde_json::from_value(serde_json::json!({
            "id": id, "source_key": "plc1.level", "source_name": "Level", "severity": "critical",
            "priority": "critical", "value": 1.0, "threshold": 0.5, "message": "High",
            "triggered_at": at, "resolved_at": null, "acknowledged_at": null, "acknowledged_by": null,
            "cooldown_until": null, "shelved_until": null, "shelved_by": null, "shelved_reason": null,
            "deadband": 0.0, "state": "active", "alarm_type": "high"
        }))
        .unwrap()
    }

    fn camera(id: i64, enabled: bool, on_alarm: bool) -> Camera {
        Camera {
            id,
            name: format!("cam{id}"),
            url: "rtsp://x/y".into(),
            enabled,
            snapshot_on_alarm: on_alarm,
            retention_hours: 24,
            created_at: 0,
        }
    }

    #[test]
    fn only_alarms_after_the_mark_are_new() {
        let alarms = vec![alarm(3, 120), alarm(1, 100), alarm(2, 110), alarm(4, 110)];
        let ids: Vec<i64> = new_alarms(&alarms, 110, 2).iter().map(|a| a.id).collect();
        assert_eq!(ids, vec![4, 3]);
        assert!(new_alarms(&alarms, 120, 3).is_empty());
    }

    #[test]
    fn cameras_capture_once_per_cooldown() {
        let cams = vec![
            camera(1, true, true),
            camera(2, false, true),
            camera(3, true, false),
            camera(4, true, true),
        ];
        let mut last = HashMap::new();
        last.insert(4, 990);
        let ids: Vec<i64> = cameras_to_capture(&cams, &last, 1000)
            .iter()
            .map(|c| c.id)
            .collect();
        assert_eq!(ids, vec![1]);
        let ids: Vec<i64> = cameras_to_capture(&cams, &last, 990 + CAMERA_COOLDOWN_SECS)
            .iter()
            .map(|c| c.id)
            .collect();
        assert_eq!(ids, vec![1, 4]);
    }
}
