use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Result};
use snmp::{SyncSession, Value};
use tokio::sync::Semaphore;
use tokio::task;
use tracing::{debug, info, warn};

use super::models::SnmpScanResult;

// ── Scan State ──────────────────────────────────────────────────────────────

/// Thread-safe scan progress tracker shared across concurrent SNMP queries.
#[derive(Clone)]
pub struct ScanState {
    pub scanning: Arc<AtomicBool>,
    pub progress: Arc<AtomicUsize>,
    pub total: Arc<AtomicUsize>,
}

impl ScanState {
    pub fn new() -> Self {
        Self {
            scanning: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(AtomicUsize::new(0)),
            total: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Returns true if a scan is currently in progress.
    pub fn is_scanning(&self) -> bool {
        self.scanning.load(Ordering::SeqCst)
    }

    /// Returns (completed, total) progress counters.
    pub fn get_progress(&self) -> (usize, usize) {
        (
            self.progress.load(Ordering::SeqCst),
            self.total.load(Ordering::SeqCst),
        )
    }
}

impl Default for ScanState {
    fn default() -> Self {
        Self::new()
    }
}

// ── OID Constants ───────────────────────────────────────────────────────────

/// sysDescr.0 — system description string
const OID_SYS_DESCR: &[u32] = &[1, 3, 6, 1, 2, 1, 1, 1, 0];
/// sysName.0 — administratively-assigned name
const OID_SYS_NAME: &[u32] = &[1, 3, 6, 1, 2, 1, 1, 5, 0];
/// sysUpTime.0 — time since network management portion was last re-initialized
const OID_SYS_UP_TIME: &[u32] = &[1, 3, 6, 1, 2, 1, 1, 3, 0];

/// Maximum number of concurrent SNMP queries per subnet scan.
const MAX_CONCURRENCY: usize = 50;

// ── CIDR Parsing ────────────────────────────────────────────────────────────

/// Parse a CIDR notation string like "192.168.1.0/24" into a network address
/// and prefix length.
///
/// # Errors
/// Returns an error if the format is invalid, the prefix is > 32, or the IP
/// address is malformed.
pub fn parse_cidr(cidr: &str) -> Result<(Ipv4Addr, u32)> {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        bail!("Invalid CIDR format: expected 'IP/PREFIX', got '{}'", cidr);
    }

    let ip: Ipv4Addr = parts[0]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid IPv4 address: '{}'", parts[0]))?;
    let prefix: u32 = parts[1]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid prefix length: '{}'", parts[1]))?;

    if prefix > 32 {
        bail!("Prefix length must be <= 32, got {}", prefix);
    }

    Ok((ip, prefix))
}

/// Enumerate all host IPs within a CIDR range, skipping the network address
/// (all-zero host bits) and the broadcast address (all-one host bits).
///
/// For /31 (point-to-point) and /32 (single host) ranges, all addresses are
/// included since there is no distinct broadcast address.
pub fn cidr_to_hosts(ip: Ipv4Addr, prefix: u32) -> Vec<Ipv4Addr> {
    if prefix >= 31 {
        // /31 = 2 hosts (point-to-point link, RFC 3021), /32 = 1 host
        // Both cases: include all addresses (no broadcast to skip)
        if prefix == 32 {
            return vec![ip];
        }
        // /31: two addresses
        let base = u32::from(ip);
        return vec![Ipv4Addr::from(base), Ipv4Addr::from(base + 1)];
    }

    let host_bits = 32 - prefix;
    let num_hosts = 2u32.pow(host_bits); // total including network + broadcast
    let base = u32::from(ip);

    // Skip network (offset 0) and broadcast (offset num_hosts - 1)
    let mut hosts = Vec::with_capacity((num_hosts - 2) as usize);
    for i in 1..num_hosts - 1 {
        hosts.push(Ipv4Addr::from(base + i));
    }
    hosts
}

// ── sysDescr Parsing ────────────────────────────────────────────────────────

/// Parse a sysDescr string to extract vendor, model, and firmware version.
///
/// Recognises common OT device patterns from Siemens, Rockwell/Allen-Bradley,
/// Schneider/Modicon, Cisco, and falls back to generic extraction.
///
/// Returns `(vendor, model, firmware)` — each may be `None` if not detected.
pub fn parse_sys_descr(sys_descr: &str) -> (Option<String>, Option<String>, Option<String>) {
    let descr = sys_descr.trim();
    if descr.is_empty() {
        return (None, None, None);
    }

    let lower = descr.to_lowercase();

    // ── Siemens ──
    if lower.contains("siemens") || lower.contains("s7-") || lower.contains("simatic") {
        return parse_siemens(descr);
    }

    // ── Rockwell / Allen-Bradley ──
    if lower.contains("rockwell")
        || lower.contains("allen-bradley")
        || lower.contains("controllogix")
        || lower.contains("compactlogix")
        || lower.contains("micrologix")
        || lower.contains("slc")
        || lower.contains("plc-5")
        || lower.contains("factorytalk")
    {
        return parse_rockwell(descr);
    }

    // ── Schneider / Modicon ──
    if lower.contains("schneider")
        || lower.contains("modicon")
        || lower.contains("quantum")
        || lower.contains("premium")
        || lower.contains("m340")
        || lower.contains("m580")
    {
        return parse_schneider(descr);
    }

    // ── Cisco ──
    if lower.contains("cisco") || lower.contains("ios ") {
        return parse_cisco(descr);
    }

    // ── Generic fallback ──
    parse_generic(descr)
}

fn parse_siemens(descr: &str) -> (Option<String>, Option<String>, Option<String>) {
    let vendor = Some("Siemens".to_string());

    // Patterns: "SIMATIC S7-1500, Firmware V2.8" or "Siemens, S7-300, HW..."
    let model = extract_word_after(descr, &["SIMATIC ", "S7-", "CPU "]);
    let firmware = extract_version(descr);

    (vendor, model, firmware)
}

fn parse_rockwell(descr: &str) -> (Option<String>, Option<String>, Option<String>) {
    let vendor = Some("Rockwell Automation".to_string());

    let model = extract_word_after(
        descr,
        &[
            "ControlLogix",
            "CompactLogix",
            "MicroLogix",
            "1756-",
            "1769-",
            "1766-",
            "1768-",
        ],
    );
    let firmware = extract_version(descr);

    (vendor, model, firmware)
}

fn parse_schneider(descr: &str) -> (Option<String>, Option<String>, Option<String>) {
    let vendor = Some("Schneider Electric".to_string());

    let model = extract_word_after(
        descr,
        &["Modicon ", "Quantum", "Premium", "M340", "M580"],
    );
    let firmware = extract_version(descr);

    (vendor, model, firmware)
}

fn parse_cisco(descr: &str) -> (Option<String>, Option<String>, Option<String>) {
    let vendor = Some("Cisco".to_string());

    let model = extract_word_after(descr, &["Cisco ", "IOS "]);
    let firmware = extract_version(descr);

    (vendor, model, firmware)
}

fn parse_generic(descr: &str) -> (Option<String>, Option<String>, Option<String>) {
    // Try to find a vendor-like word (capitalised, early in the string)
    let vendor = descr
        .split(|c: char| c == ',' || c == '\n' || c == ';')
        .next()
        .and_then(|s| s.split_whitespace().next())
        .filter(|w| w.len() > 1 && w.chars().next().map_or(false, |c| c.is_uppercase()))
        .map(|w| w.trim_end_matches(',').to_string());

    let firmware = extract_version(descr);

    (vendor, None, firmware)
}

/// Extract the first word found immediately after one of the given markers.
fn extract_word_after(text: &str, markers: &[&str]) -> Option<String> {
    let lower = text.to_lowercase();
    for marker in markers {
        if let Some(idx) = lower.find(&marker.to_lowercase()) {
            let start = idx + marker.len();
            let rest = text[start..].trim_start();
            // Take the first word (may include hyphens for model numbers)
            let word: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '.')
                .collect();
            if !word.is_empty() {
                return Some(word.trim_end_matches('-').trim_end_matches('.').to_string());
            }
        }
    }
    None
}

/// Try to extract a version string from the description.
fn extract_version(text: &str) -> Option<String> {
    let lower = text.to_lowercase();

    // Look for common version patterns
    let patterns = [
        "firmware v",
        "firmware version ",
        "version ",
        "v",
        "sw version ",
        "software version ",
        "hw: ",
        "fw: ",
    ];

    for pat in &patterns {
        if let Some(idx) = lower.find(pat) {
            let start = idx + pat.len();
            let rest = text[start..].trim_start();
            let version: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
                .collect();
            // Validate it looks like a version (starts with a digit)
            if version.starts_with(|c: char| c.is_ascii_digit()) && version.len() >= 2 {
                return Some(version);
            }
        }
    }

    None
}

// ── SNMP Host Query ─────────────────────────────────────────────────────────

/// Query a single host via SNMPv2c for sysDescr, sysName, and sysUpTime.
///
/// Returns `Some(SnmpScanResult)` on success, `None` if the host does not
/// respond or returns an error.
///
/// This function performs blocking I/O (the `snmp` crate is synchronous) and
/// is designed to be called from a `tokio::task::spawn_blocking` context.
pub fn snmp_query_host(
    ip: Ipv4Addr,
    community: &str,
    timeout_ms: u32,
) -> Option<SnmpScanResult> {
    let addr = format!("{}:161", ip);
    let timeout = Duration::from_millis(timeout_ms as u64);

    // Build the SNMP session
    let mut sess = match SyncSession::new(&*addr, community.as_bytes(), Some(timeout), 0) {
        Ok(s) => s,
        Err(e) => {
            debug!(%ip, error = %e, "SNMP session creation failed");
            return None;
        }
    };

    // Query sysDescr
    let sys_descr = match sess.get(OID_SYS_DESCR) {
        Ok(mut resp) => resp
            .varbinds
            .next()
            .and_then(|(_oid, val)| match val {
                Value::OctetString(bytes) => {
                    Some(String::from_utf8_lossy(bytes).into_owned())
                }
                _ => None,
            }),
        Err(e) => {
            debug!(%ip, error = ?e, "SNMP get sysDescr failed");
            return None;
        }
    };

    // Query sysName
    let sys_name = sess
        .get(OID_SYS_NAME)
        .ok()
        .and_then(|mut resp| {
            resp.varbinds
                .next()
                .and_then(|(_oid, val)| match val {
                    Value::OctetString(bytes) => {
                        Some(String::from_utf8_lossy(bytes).into_owned())
                    }
                    _ => None,
                })
        });

    // Query sysUpTime (Timeticks value in hundredths of a second)
    let sys_uptime = sess
        .get(OID_SYS_UP_TIME)
        .ok()
        .and_then(|mut resp| {
            resp.varbinds
                .next()
                .and_then(|(_oid, val)| match val {
                    Value::Timeticks(ticks) => Some(format_timeticks(ticks)),
                    Value::Integer(n) => Some(format_timeticks(n as u32)),
                    _ => None,
                })
        });

    let sys_descr_str = sys_descr.unwrap_or_default();
    let (vendor, model, firmware) = parse_sys_descr(&sys_descr_str);

    Some(SnmpScanResult {
        ip: ip.to_string(),
        sys_descr: if sys_descr_str.is_empty() {
            None
        } else {
            Some(sys_descr_str)
        },
        sys_name,
        sys_uptime,
        vendor,
        model,
        firmware,
        mac_address: None, // MAC not available via standard SNMP OIDs without ifTable walk
        scanned_at: chrono_free_timestamp(),
    })
}

/// Format timeticks (hundredths of a second) into a human-readable uptime
/// string like "3d 14h 22m 11s".
fn format_timeticks(ticks: u32) -> String {
    let total_secs = ticks as u64 / 100;
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    parts.push(format!("{}h", hours));
    parts.push(format!("{}m", minutes));
    parts.push(format!("{}s", seconds));
    parts.join(" ")
}

/// Generate an ISO-8601-ish timestamp without pulling in `chrono`.
fn chrono_free_timestamp() -> String {
    // Use `time` crate which is already a dependency
    use time::OffsetDateTime;
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string())
}

// ── Scan Runner ─────────────────────────────────────────────────────────────

/// Run an SNMP scan across a subnet, querying each host concurrently with a
/// semaphore limit of 50.
///
/// Returns an error if a scan is already in progress (per the `ScanState`).
/// On success, returns all hosts that responded to SNMP queries.
pub async fn run_scan(
    subnet: &str,
    community: &str,
    timeout_ms: u32,
    scan_state: &ScanState,
) -> Result<Vec<SnmpScanResult>> {
    // Reject concurrent scans
    if scan_state.is_scanning() {
        bail!("SNMP scan already in progress");
    }

    // Parse CIDR and enumerate hosts
    let (network, prefix) = parse_cidr(subnet)?;
    let hosts = cidr_to_hosts(network, prefix);
    let total = hosts.len();

    if total == 0 {
        return Ok(Vec::new());
    }

    info!(
        subnet = subnet,
        total_hosts = total,
        community = %community,
        "Starting SNMP scan"
    );

    // Initialise state
    scan_state.scanning.store(true, Ordering::SeqCst);
    scan_state.total.store(total, Ordering::SeqCst);
    scan_state.progress.store(0, Ordering::SeqCst);

    // Semaphore to cap concurrency at MAX_CONCURRENCY
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENCY));
    let progress = Arc::clone(&scan_state.progress);
    let community_owned = community.to_string();

    // Spawn blocking tasks for each host (snmp crate is sync)
    let mut handles = Vec::with_capacity(total);

    for ip in hosts {
        let permit = semaphore.clone().acquire_owned().await?;
        let progress = Arc::clone(&progress);
        let comm = community_owned.clone();

        let handle = task::spawn_blocking(move || {
            let result = snmp_query_host(ip, &comm, timeout_ms);
            drop(permit);
            progress.fetch_add(1, Ordering::SeqCst);
            result
        });
        handles.push(handle);
    }

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(Some(scan_result)) => results.push(scan_result),
            Ok(None) => { /* host did not respond — expected for most IPs */ }
            Err(e) => {
                warn!(error = %e, "SNMP scan task panicked or was cancelled");
            }
        }
    }

    scan_state.scanning.store(false, Ordering::SeqCst);

    info!(
        subnet = subnet,
        responding = results.len(),
        total_queried = total,
        "SNMP scan complete"
    );

    Ok(results)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cidr_valid() {
        let (ip, prefix) = parse_cidr("192.168.1.0/24").unwrap();
        assert_eq!(ip, Ipv4Addr::new(192, 168, 1, 0));
        assert_eq!(prefix, 24);
    }

    #[test]
    fn test_parse_cidr_invalid_format() {
        assert!(parse_cidr("192.168.1.0").is_err());
        assert!(parse_cidr("192.168.1.0/24/extra").is_err());
        assert!(parse_cidr("not-an-ip/24").is_err());
        assert!(parse_cidr("192.168.1.0/33").is_err());
    }

    #[test]
    fn test_cidr_to_hosts_small() {
        // /30 has 4 addresses: network, 2 usable hosts, broadcast
        let hosts = cidr_to_hosts(Ipv4Addr::new(192, 168, 1, 0), 30);
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0], Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(hosts[1], Ipv4Addr::new(192, 168, 1, 2));
    }

    #[test]
    fn test_cidr_to_hosts_32() {
        let hosts = cidr_to_hosts(Ipv4Addr::new(10, 0, 0, 1), 32);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0], Ipv4Addr::new(10, 0, 0, 1));
    }

    #[test]
    fn test_cidr_to_hosts_31() {
        // /31 point-to-point: both addresses are usable
        let hosts = cidr_to_hosts(Ipv4Addr::new(10, 0, 0, 0), 31);
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0], Ipv4Addr::new(10, 0, 0, 0));
        assert_eq!(hosts[1], Ipv4Addr::new(10, 0, 0, 1));
    }

    #[test]
    fn test_cidr_to_hosts_24() {
        let hosts = cidr_to_hosts(Ipv4Addr::new(192, 168, 1, 0), 24);
        assert_eq!(hosts.len(), 254);
        assert_eq!(hosts[0], Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(hosts[253], Ipv4Addr::new(192, 168, 1, 254));
    }

    #[test]
    fn test_parse_sys_descr_siemens() {
        let (vendor, model, firmware) = parse_sys_descr(
            "SIMATIC S7-1500, Firmware V2.8.1, Serial: XYZ",
        );
        assert_eq!(vendor.as_deref(), Some("Siemens"));
        assert!(model.is_some());
        assert!(firmware.is_some());
    }

    #[test]
    fn test_parse_sys_descr_rockwell() {
        let (vendor, _model, _firmware) =
            parse_sys_descr("Rockwell Automation/1756-L83E/Firmware 33.11");
        assert_eq!(vendor.as_deref(), Some("Rockwell Automation"));
    }

    #[test]
    fn test_parse_sys_descr_schneider() {
        let (vendor, _model, _firmware) =
            parse_sys_descr("Schneider Electric, Modicon M580, Firmware V3.20");
        assert_eq!(vendor.as_deref(), Some("Schneider Electric"));
    }

    #[test]
    fn test_parse_sys_descr_cisco() {
        let (vendor, _model, _firmware) =
            parse_sys_descr("Cisco IOS Software, Version 15.2(4)M7");
        assert_eq!(vendor.as_deref(), Some("Cisco"));
    }

    #[test]
    fn test_parse_sys_descr_empty() {
        let (vendor, model, firmware) = parse_sys_descr("");
        assert!(vendor.is_none());
        assert!(model.is_none());
        assert!(firmware.is_none());
    }

    #[test]
    fn test_format_timeticks() {
        assert_eq!(format_timeticks(0), "0h 0m 0s");
        assert_eq!(format_timeticks(100), "0h 0m 1s");
        // 1 day = 8640000 hundredths of a second
        assert_eq!(format_timeticks(8640000), "1d 0h 0m 0s");
    }

    #[test]
    fn test_scan_state() {
        let state = ScanState::new();
        assert!(!state.is_scanning());
        assert_eq!(state.get_progress(), (0, 0));
    }
}
