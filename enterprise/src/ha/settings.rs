//! The `ha:` section of the config file, validated.

use std::time::Duration;

use anyhow::{Result, bail};
use voltrus::models::{HaConfig, HaWitnessConfig};

use super::link::MIN_SECRET_LEN;

/// How a node with no recorded history picks its role. After the first start
/// the recorded role rules and this is not consulted again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitialRole {
    Primary,
    Standby,
    /// Probe the peer: follow it if it is primary, otherwise take the active
    /// role (the lower node id wins when both start at once).
    Auto,
}

#[derive(Debug, Clone)]
pub struct HaSettings {
    pub node_id: String,
    pub initial_role: InitialRole,
    pub bind_address: String,
    pub peer_address: String,
    pub heartbeat: Duration,
    pub missed: u32,
    pub sync_interval: Duration,
    pub secret: String,
    pub tls_files: Option<(String, String)>,
    pub standby_polling: bool,
    pub witness: Option<HaWitnessConfig>,
}

impl HaSettings {
    /// A silent primary is declared dead after this long.
    pub fn failover_timeout(&self) -> Duration {
        self.heartbeat * self.missed
    }

    pub fn from_config(cfg: &HaConfig) -> Result<Self> {
        if cfg.node_id.is_empty()
            || cfg.node_id.len() > 128
            || cfg.node_id.contains(char::is_whitespace)
        {
            bail!("ha.node_id must be 1 to 128 characters without spaces");
        }
        let initial_role = match cfg.role.as_str() {
            "primary" => InitialRole::Primary,
            "standby" => InitialRole::Standby,
            "auto" => InitialRole::Auto,
            other => bail!("ha.role must be primary, standby or auto, not '{other}'"),
        };
        if cfg.bind_address.is_empty() {
            bail!("ha.bind_address is required");
        }
        if cfg.peer_address.is_empty() {
            bail!("ha.peer_address is required");
        }
        if !(100..=60_000).contains(&cfg.heartbeat_interval_ms) {
            bail!("ha.heartbeat_interval_ms must be between 100 and 60000");
        }
        if !(2..=100).contains(&cfg.failover_after_missed) {
            bail!("ha.failover_after_missed must be between 2 and 100");
        }
        if !(20..=10_000).contains(&cfg.sync_interval_ms) {
            bail!("ha.sync_interval_ms must be between 20 and 10000");
        }
        let secret = std::env::var("VOLTRUS_HA_SECRET")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| cfg.shared_secret.clone());
        if secret.len() < MIN_SECRET_LEN {
            bail!(
                "ha.shared_secret (or VOLTRUS_HA_SECRET) must be at least {MIN_SECRET_LEN} characters, the same on both nodes (try: openssl rand -hex 32)"
            );
        }
        let tls_files = match &cfg.tls {
            Some(t) if t.enabled => {
                if t.cert_path.is_empty() || t.key_path.is_empty() {
                    bail!("ha.tls needs cert_path and key_path when enabled");
                }
                Some((t.cert_path.clone(), t.key_path.clone()))
            }
            _ => None,
        };
        if let Some(w) = &cfg.witness {
            if !["arbiter", "tcp", "http"].contains(&w.kind.as_str()) {
                bail!("ha.witness.kind must be arbiter, tcp or http");
            }
            if w.address.is_empty() {
                bail!("ha.witness.address is required");
            }
        }
        Ok(Self {
            node_id: cfg.node_id.clone(),
            initial_role,
            bind_address: cfg.bind_address.clone(),
            peer_address: cfg.peer_address.clone(),
            heartbeat: Duration::from_millis(cfg.heartbeat_interval_ms),
            missed: cfg.failover_after_missed,
            sync_interval: Duration::from_millis(cfg.sync_interval_ms),
            secret,
            tls_files,
            standby_polling: cfg.standby_polling,
            witness: cfg.witness.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> HaConfig {
        HaConfig {
            enabled: true,
            role: "standby".into(),
            node_id: "node-b".into(),
            bind_address: "0.0.0.0:7901".into(),
            peer_address: "10.0.0.1:7901".into(),
            shared_secret: "x".repeat(40),
            ..Default::default()
        }
    }

    #[test]
    fn a_complete_config_validates_and_derives_the_failover_time() {
        let s = HaSettings::from_config(&good()).unwrap();
        assert_eq!(s.failover_timeout(), Duration::from_secs(3));
        assert_eq!(s.initial_role, InitialRole::Standby);
    }

    #[test]
    fn unsafe_or_incomplete_configs_are_refused() {
        let bad = |f: fn(&mut HaConfig)| {
            let mut c = good();
            f(&mut c);
            assert!(HaSettings::from_config(&c).is_err());
        };
        bad(|c| c.shared_secret = "short".into());
        bad(|c| c.role = "master".into());
        bad(|c| c.node_id = "has space".into());
        bad(|c| c.peer_address.clear());
        bad(|c| c.failover_after_missed = 1);
        bad(|c| c.heartbeat_interval_ms = 10);
        bad(|c| {
            c.witness = Some(HaWitnessConfig {
                kind: "magic".into(),
                address: "x".into(),
                timeout_ms: 1000,
            })
        });
    }
}
