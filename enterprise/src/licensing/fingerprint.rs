#![allow(dead_code)]
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Fingerprint {
    pub hostname: String,
    pub cpu: String,
    pub disk_serial: String,
    pub mac: String,
    pub machine_id: String,
}

impl Fingerprint {
    pub const MIN_MATCH: usize = 3;
    pub const TOTAL_COMPONENTS: usize = 5;

    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.hostname.as_bytes());
        hasher.update(b"\x00");
        hasher.update(self.cpu.as_bytes());
        hasher.update(b"\x00");
        hasher.update(self.disk_serial.as_bytes());
        hasher.update(b"\x00");
        hasher.update(self.mac.as_bytes());
        hasher.update(b"\x00");
        hasher.update(self.machine_id.as_bytes());
        format!("sha256:{}", hex::encode(hasher.finalize()))
    }

    pub fn match_score(&self, other: &Fingerprint) -> usize {
        let mut score = 0;
        if self.hostname == other.hostname {
            score += 1;
        }
        if self.cpu == other.cpu {
            score += 1;
        }
        if self.disk_serial == other.disk_serial {
            score += 1;
        }
        if self.mac == other.mac {
            score += 1;
        }
        if self.machine_id == other.machine_id {
            score += 1;
        }
        score
    }

    pub fn to_components_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("hostname".to_string(), self.hostname.clone());
        map.insert("cpu".to_string(), self.cpu.clone());
        map.insert("disk_serial".to_string(), self.disk_serial.clone());
        map.insert("mac".to_string(), self.mac.clone());
        map.insert("machine_id".to_string(), self.machine_id.clone());
        map
    }
}

pub fn collect_fingerprint() -> Fingerprint {
    Fingerprint {
        hostname: get_hostname(),
        cpu: get_cpu_info(),
        disk_serial: get_disk_serial(),
        mac: get_mac_address(),
        machine_id: get_machine_id(),
    }
}

fn get_hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .or_else(|_| {
            std::process::Command::new("hostname")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .ok_or(std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

fn get_cpu_info() -> String {
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_all();
    sys.cpus()
        .first()
        .map(|cpu| cpu.brand().trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_disk_serial() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("diskutil")
            .args(["info", "-all"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("Device Identifier")
                    && let Some(id) = line.split(':').nth(1)
                {
                    return id.trim().to_string();
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/block") {
            let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
            paths.sort();
            for path in paths {
                let serial_path = path.join("serial");
                if let Ok(serial) = std::fs::read_to_string(&serial_path) {
                    let serial = serial.trim();
                    if !serial.is_empty() {
                        return serial.to_string();
                    }
                }
            }
        }
    }

    "unknown".to_string()
}

/// Interface name prefixes of virtual adapters (containers, bridges, VPNs): their MAC
/// changes with software state, so they never identify the machine.
const VIRTUAL_INTERFACE_PREFIXES: &[&str] = &[
    "lo",
    "veth",
    "docker",
    "br-",
    "virbr",
    "vnet",
    "tun",
    "tap",
    "tailscale",
    "zt",
    "wg",
    "loopback",
    "awdl",
    "llw",
    "bridge",
    "utun",
    "gif",
    "stf",
    "anpi",
    "ap",
];

/// MAC of the first physical-looking interface, by name. The interface list comes out
/// of a hash map in arbitrary order, so it is sorted: the same machine must produce the
/// same fingerprint on every run (the activation file is bound to its hash).
fn get_mac_address() -> String {
    use sysinfo::Networks;
    let networks = Networks::new_with_refreshed_list();
    let mut interfaces: Vec<_> = networks.iter().collect();
    interfaces.sort_by(|a, b| a.0.cmp(b.0));
    for (name, network) in interfaces {
        if VIRTUAL_INTERFACE_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(prefix))
        {
            continue;
        }
        let mac_str = network.mac_address().to_string();
        if mac_str != "00:00:00:00:00:00" && !mac_str.is_empty() {
            return mac_str;
        }
    }
    "unknown".to_string()
}

fn get_machine_id() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            return id.trim().to_string();
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("IOPlatformUUID")
                    && let Some(start) = line.find('"')
                    && let Some(end) = line[start + 1..].find('"')
                {
                    return line[start + 1..start + 1 + end].to_string();
                }
            }
        }
    }

    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collected_fingerprint_is_stable_across_runs() {
        // MAC and disk serial come from unordered system lists: the hash an activation
        // is bound to must not depend on iteration order.
        let first = collect_fingerprint().hash();
        for _ in 0..5 {
            assert_eq!(collect_fingerprint().hash(), first);
        }
    }

    #[test]
    fn test_fingerprint_hash_consistency() {
        let fp = Fingerprint {
            hostname: "test".to_string(),
            cpu: "intel".to_string(),
            disk_serial: "abc".to_string(),
            mac: "aa:bb:cc:dd:ee:ff".to_string(),
            machine_id: "123".to_string(),
        };
        let hash1 = fp.hash();
        let hash2 = fp.hash();
        assert_eq!(hash1, hash2);
        assert!(hash1.starts_with("sha256:"));
    }

    #[test]
    fn test_fingerprint_match_score() {
        let fp1 = Fingerprint {
            hostname: "a".to_string(),
            cpu: "b".to_string(),
            disk_serial: "c".to_string(),
            mac: "d".to_string(),
            machine_id: "e".to_string(),
        };
        let fp2 = Fingerprint {
            hostname: "a".to_string(),
            cpu: "b".to_string(),
            disk_serial: "x".to_string(),
            mac: "y".to_string(),
            machine_id: "e".to_string(),
        };
        assert_eq!(fp1.match_score(&fp2), 3);
    }
}
