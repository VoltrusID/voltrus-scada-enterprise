use std::sync::Arc;

use anyhow::{Context, Result};
use dashmap::DashMap;

use crate::models::{Camera, CameraSnapshot};

use super::client::RtspClient;

// ── Stream Proxy ──

pub(crate) struct CameraState {
    url: String,
    last_snapshot: Option<CameraSnapshot>,
}

pub struct StreamProxy {
    pub(crate) cameras: Arc<DashMap<i64, CameraState>>,
    pub(crate) snapshot_dir: String,
}

impl StreamProxy {
    pub fn new(snapshot_dir: &str) -> Self {
        // Ensure the snapshot directory exists
        let _ = std::fs::create_dir_all(snapshot_dir);
        StreamProxy {
            cameras: Arc::new(DashMap::new()),
            snapshot_dir: snapshot_dir.to_string(),
        }
    }

    pub async fn add_camera(&self, camera: &Camera) {
        self.cameras.insert(
            camera.id,
            CameraState {
                url: camera.url.clone(),
                last_snapshot: None,
            },
        );
    }

    pub async fn remove_camera(&self, camera_id: i64) {
        self.cameras.remove(&camera_id);
    }

    /// Capture a snapshot from the specified camera.
    pub async fn capture_snapshot(
        &self,
        camera: &Camera,
        alarm_source: Option<&str>,
    ) -> Result<CameraSnapshot> {
        let mut client = RtspClient::new(&camera.url);
        let data = client.capture_snapshot().await?;

        let timestamp = crate::timeutil::now_secs();
        let filename = format!("cam{}_{}.jpg", camera.id, timestamp);
        let file_path = format!("{}/{}", self.snapshot_dir.trim_end_matches('/'), filename);

        std::fs::write(&file_path, &data)
            .with_context(|| format!("Failed to write snapshot to {}", file_path))?;

        let file_size = data.len() as u64;

        let snapshot = CameraSnapshot {
            id: 0, // Will be assigned by storage
            camera_id: camera.id,
            timestamp,
            alarm_source: alarm_source.map(|s| s.to_string()),
            file_path: file_path.clone(),
            file_size,
        };

        // Update camera state
        if let Some(mut state) = self.cameras.get_mut(&camera.id) {
            state.last_snapshot = Some(snapshot.clone());
        }

        Ok(snapshot)
    }

    pub fn get_stream_url(&self, camera_id: i64) -> Option<String> {
        self.cameras.get(&camera_id).map(|state| state.url.clone())
    }
}
