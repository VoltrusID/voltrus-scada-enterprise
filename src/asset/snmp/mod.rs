pub mod client;
pub mod scanner;

// Re-export public API so external `crate::asset::snmp::{...}` keeps working.
#[allow(unused_imports)]
pub use client::{snmp_query_host, OID_SYS_DESCR, OID_SYS_NAME, OID_SYS_UP_TIME};
pub use scanner::{run_scan, ScanState};

// Available for future use:
// pub use client::{chrono_free_timestamp, format_timeticks, parse_sys_descr};
// pub use scanner::{cidr_to_hosts, parse_cidr};
