use anyhow::{Context, Result};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AssetStorage {
    conn: Arc<Mutex<Connection>>,
}

impl AssetStorage {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS assets (
                id TEXT PRIMARY KEY,
                device_ref TEXT UNIQUE,
                name TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                vendor TEXT,
                model TEXT,
                firmware_version TEXT,
                serial_number TEXT,
                ip_address TEXT,
                mac_address TEXT,
                location TEXT,
                criticality TEXT NOT NULL DEFAULT 'medium',
                tags TEXT DEFAULT '[]',
                risk_score INTEGER DEFAULT 0,
                last_maintenance TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_assets_device_ref ON assets(device_ref);
            CREATE INDEX IF NOT EXISTS idx_assets_ip ON assets(ip_address);
            CREATE INDEX IF NOT EXISTS idx_assets_type ON assets(asset_type);
            CREATE INDEX IF NOT EXISTS idx_assets_criticality ON assets(criticality);

            CREATE TABLE IF NOT EXISTS maintenance_logs (
                id TEXT PRIMARY KEY,
                asset_id TEXT NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
                description TEXT NOT NULL,
                technician TEXT NOT NULL,
                performed_at TEXT NOT NULL,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_maintenance_asset_id ON maintenance_logs(asset_id);

            CREATE TABLE IF NOT EXISTS firmware_catalog (
                id TEXT PRIMARY KEY,
                vendor TEXT NOT NULL,
                model TEXT NOT NULL,
                latest_version TEXT NOT NULL,
                release_date TEXT,
                notes TEXT,
                updated_at TEXT NOT NULL,
                UNIQUE(vendor, model)
            );

            CREATE TABLE IF NOT EXISTS snmp_scan_results (
                ip TEXT PRIMARY KEY,
                sys_descr TEXT,
                sys_name TEXT,
                sys_uptime INTEGER,
                vendor TEXT,
                model TEXT,
                firmware TEXT,
                mac_address TEXT,
                scanned_at TEXT NOT NULL
            );",
        )
        .context("Failed to create asset tables")?;

        Ok(())
    }
}
