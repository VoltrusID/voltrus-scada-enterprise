pub mod client;
pub mod scanner;

// Re-export public API so external `crate::asset::snmp::{...}` keeps working.
pub use client::{
    chrono_free_timestamp, format_timeticks, parse_sys_descr, snmp_query_host, OID_SYS_DESCR,
    OID_SYS_NAME, OID_SYS_UP_TIME,
};
pub use scanner::{cidr_to_hosts, parse_cidr, run_scan, ScanState};
