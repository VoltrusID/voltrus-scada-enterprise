use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::models::{
    DataPoint, Device, ScreenLayoutRow, DataSource, Setting, TwinLayout, TwinConnection, User, UserShortcut,
    NavShortcut, Alarm, CanvasLayoutItem,
    ModbusDevice, ModbusRegister, ModbusRegisterInput, DeviceConfig,
    ScadaEquipment, ScadaConnection,
    Flow, FlowRun,
    OpcUaDevice, OpcUaNode, OpcUaDeviceConfig, OpcUaNodeConfig,
    OpcUaSecurityPolicy, OpcUaSecurityMode,
};

#[derive(Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
    db_path: String,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        Self::new_with_key(path, None)
    }

    pub fn new_with_key(path: &str, encryption_key: Option<&str>) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database: {path}"))?;

        // Apply SQLCipher encryption key if provided
        if let Some(key) = encryption_key {
            if let Err(e) = conn.execute_batch(&format!("PRAGMA key = '{}';", key.replace("'", "''"))) {
                tracing::warn!(
                    "Failed to set database encryption key (SQLCipher not available?): {}. Continuing without encryption.",
                    e
                );
            } else {
                tracing::info!("Database encryption key applied successfully");
            }
        }

        init_schema(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: path.to_string(),
        })
    }

    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to open in-memory DB")?;
        init_schema(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: ":memory:".to_string(),
        })
    }

    pub fn db_path(&self) -> &str {
        &self.db_path
    }

    pub async fn insert_batch(&self, records: &[(String, i64, String, f64)]) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        let conn = self.conn.clone();
        let records = records.to_vec();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            for (device_id, timestamp, metric, value) in &records {
                tx.execute(
                    "INSERT INTO telemetry (device_id, timestamp, metric, value) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![device_id, timestamp, metric, value],
                )
                .context("Insert failed")?;
            }
            tx.commit().context("Commit failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn query_history(
        &self,
        device_id: &str,
        metric: &str,
        from: i64,
        to: i64,
    ) -> Result<Vec<DataPoint>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        let metric = metric.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT timestamp, value FROM telemetry
                     WHERE device_id = ?1 AND metric = ?2
                     AND timestamp >= ?3 AND timestamp <= ?4
                     ORDER BY timestamp
                     LIMIT 10000",
                )
                .context("Prepare failed")?;

            let rows = stmt
                .query_map(
                    rusqlite::params![device_id, metric, from, to],
                    |row| {
                        Ok(DataPoint {
                            timestamp: row.get(0)?,
                            value: row.get(1)?,
                        })
                    },
                )
                .context("Query failed")?;

            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    /// Query all metrics for a device within a time range.
    /// Returns a map of timestamp → (metric → value).
    pub async fn query_device_metrics(
        &self,
        device_id: &str,
        from: i64,
        to: i64,
    ) -> Result<Vec<(i64, std::collections::HashMap<String, f64>)>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT timestamp, metric, value FROM telemetry
                     WHERE device_id = ?1
                     AND timestamp >= ?2 AND timestamp <= ?3
                     ORDER BY timestamp
                     LIMIT 10000",
                )
                .context("Prepare failed")?;

            let rows = stmt
                .query_map(
                    rusqlite::params![device_id, from, to],
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, f64>(2)?,
                        ))
                    },
                )
                .context("Query failed")?;

            let mut grouped: std::collections::BTreeMap<i64, std::collections::HashMap<String, f64>> =
                std::collections::BTreeMap::new();
            for row in rows {
                let (ts, metric, value) = row.context("Row mapping failed")?;
                grouped.entry(ts).or_default().insert(metric, value);
            }

            Ok(grouped.into_iter().collect())
        })
        .await
        .context("Task join failed")?
    }

    // ── Downsampling & Retention ──

    pub async fn downsample_raw_to_hourly(&self, cutoff: i64) -> Result<u64> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO telemetry_hourly (device_id, metric, bucket_start, avg_value, min_value, max_value, sample_count)
                 SELECT device_id, metric,
                        (timestamp / 3600) * 3600 as bucket_start,
                        AVG(value), MIN(value), MAX(value), COUNT(*)
                 FROM telemetry
                 WHERE timestamp < ?1
                 GROUP BY device_id, metric, bucket_start",
                rusqlite::params![cutoff],
            ).context("Downsample raw to hourly failed").map(|n| n as u64)
        }).await.context("Task join failed")?
    }

    pub async fn downsample_hourly_to_daily(&self, cutoff: i64) -> Result<u64> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO telemetry_daily (device_id, metric, bucket_start, avg_value, min_value, max_value, sample_count)
                 SELECT device_id, metric,
                        (bucket_start / 86400) * 86400 as day_start,
                        AVG(avg_value), MIN(min_value), MAX(max_value), SUM(sample_count)
                 FROM telemetry_hourly
                 WHERE bucket_start < ?1
                 GROUP BY device_id, metric, day_start",
                rusqlite::params![cutoff],
            ).context("Downsample hourly to daily failed").map(|n| n as u64)
        }).await.context("Task join failed")?
    }

    pub async fn delete_raw_older_than(&self, cutoff: i64) -> Result<u64> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "DELETE FROM telemetry WHERE timestamp < ?1",
                rusqlite::params![cutoff],
            ).context("Delete raw telemetry failed").map(|n| n as u64)
        }).await.context("Task join failed")?
    }

    pub async fn delete_hourly_older_than(&self, cutoff: i64) -> Result<u64> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "DELETE FROM telemetry_hourly WHERE bucket_start < ?1",
                rusqlite::params![cutoff],
            ).context("Delete hourly telemetry failed").map(|n| n as u64)
        }).await.context("Task join failed")?
    }

    pub async fn delete_daily_older_than(&self, cutoff: i64) -> Result<u64> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "DELETE FROM telemetry_daily WHERE bucket_start < ?1",
                rusqlite::params![cutoff],
            ).context("Delete daily telemetry failed").map(|n| n as u64)
        }).await.context("Task join failed")?
    }

    pub async fn query_raw_for_archive(&self, cutoff: i64) -> Result<Vec<(String, String, i64, f64)>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT device_id, metric, timestamp, value FROM telemetry
                 WHERE timestamp < ?1 ORDER BY timestamp"
            ).context("Prepare archive query failed")?;
            let rows = stmt.query_map(rusqlite::params![cutoff], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            }).context("Query archive rows failed")?;
            rows.collect::<Result<Vec<_>, _>>().context("Row mapping failed")
        }).await.context("Task join failed")?
    }

    /// Query telemetry across all retention tiers, transparently merging raw/hourly/daily data.
    ///
    /// Tier time windows:
    /// - Raw:   [raw_cutoff, ∞)  — recent high-resolution data
    /// - Hourly: [hourly_cutoff, raw_cutoff) — mid-range aggregates
    /// - Daily:  (-∞, hourly_cutoff) — long-term aggregates
    ///
    /// `min_interval_secs` controls resolution awareness: if the caller needs data
    /// at coarser intervals than a tier provides, that tier is skipped.
    /// (raw=5s, hourly=3600s, daily=86400s)
    pub async fn query_device_metrics_merged(
        &self,
        device_id: &str,
        from: i64,
        to: i64,
        raw_cutoff: i64,
        hourly_cutoff: i64,
        min_interval_secs: i64,
    ) -> Result<Vec<(i64, std::collections::HashMap<String, f64>)>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let mut grouped: std::collections::BTreeMap<i64, std::collections::HashMap<String, f64>> =
                std::collections::BTreeMap::new();

            // Daily data: (-∞, hourly_cutoff)
            // Always useful at any resolution
            let daily_from = from;
            let daily_to = to.min(hourly_cutoff);
            if daily_from < daily_to {
                let mut stmt = conn.prepare(
                    "SELECT bucket_start, metric, avg_value FROM telemetry_daily
                     WHERE device_id = ?1 AND bucket_start >= ?2 AND bucket_start <= ?3
                     ORDER BY bucket_start LIMIT 10000"
                ).context("Prepare daily failed")?;
                let rows = stmt.query_map(rusqlite::params![device_id, daily_from, daily_to], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, f64>(2)?))
                }).context("Query daily failed")?;
                for row in rows {
                    let (ts, metric, value) = row.context("Row mapping failed")?;
                    grouped.entry(ts).or_default().insert(metric, value);
                }
            }

            // Hourly data: [hourly_cutoff, raw_cutoff)
            // Skip if caller needs daily+ resolution only
            let hourly_from = from.max(hourly_cutoff);
            let hourly_to = to.min(raw_cutoff);
            if min_interval_secs <= 3600 && hourly_from < hourly_to {
                let mut stmt = conn.prepare(
                    "SELECT bucket_start, metric, avg_value FROM telemetry_hourly
                     WHERE device_id = ?1 AND bucket_start >= ?2 AND bucket_start <= ?3
                     ORDER BY bucket_start LIMIT 10000"
                ).context("Prepare hourly failed")?;
                let rows = stmt.query_map(rusqlite::params![device_id, hourly_from, hourly_to], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, f64>(2)?))
                }).context("Query hourly failed")?;
                for row in rows {
                    let (ts, metric, value) = row.context("Row mapping failed")?;
                    grouped.entry(ts).or_default().insert(metric, value);
                }
            }

            // Raw data: [raw_cutoff, ∞)
            // Skip if caller needs hourly+ resolution only (raw is ~5s intervals)
            let raw_from = from.max(raw_cutoff);
            let raw_to = to;
            if min_interval_secs <= 300 && raw_from < raw_to {
                let mut stmt = conn.prepare(
                    "SELECT timestamp, metric, value FROM telemetry
                     WHERE device_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3
                     ORDER BY timestamp LIMIT 10000"
                ).context("Prepare raw failed")?;
                let rows = stmt.query_map(rusqlite::params![device_id, raw_from, raw_to], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, f64>(2)?))
                }).context("Query raw failed")?;
                for row in rows {
                    let (ts, metric, value) = row.context("Row mapping failed")?;
                    grouped.entry(ts).or_default().insert(metric, value);
                }
            }

            Ok(grouped.into_iter().collect())
        }).await.context("Task join failed")?
    }

    /// Count total rows matching the export query filters (for pagination meta).
    pub async fn count_telemetry(
        &self,
        device_id: Option<&str>,
        metric: Option<&str>,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let device_id = device_id.map(|s| s.to_string());
        let metric = metric.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut sql = String::from("SELECT COUNT(*) FROM telemetry WHERE 1=1");
            if device_id.is_some() {
                sql.push_str(" AND device_id = ?1");
            }
            if metric.is_some() {
                sql.push_str(&format!(" AND metric = ?{}", if device_id.is_some() { 2 } else { 1 }));
            }
            if from.is_some() {
                let idx = device_id.is_some() as usize + metric.is_some() as usize + 1;
                sql.push_str(&format!(" AND timestamp >= ?{idx}"));
            }
            if to.is_some() {
                let idx = device_id.is_some() as usize + metric.is_some() as usize + from.is_some() as usize + 1;
                sql.push_str(&format!(" AND timestamp <= ?{idx}"));
            }

            let mut stmt = conn.prepare(&sql).context("Prepare count failed")?;

            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            if let Some(ref d) = device_id {
                params.push(Box::new(d.clone()));
            }
            if let Some(ref m) = metric {
                params.push(Box::new(m.clone()));
            }
            if let Some(f) = from {
                params.push(Box::new(f));
            }
            if let Some(t) = to {
                params.push(Box::new(t));
            }

            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let count: i64 = stmt.query_row(param_refs.as_slice(), |row| row.get(0))
                .context("Count query failed")?;
            Ok(count)
        }).await.context("Task join failed")?
    }

    /// Query telemetry rows for the export API with optional aggregation.
    ///
    /// When `aggregation` is None, returns raw rows from the `telemetry` table.
    /// When `aggregation` is Some("avg"|"min"|"max"|"count"|"sum"), uses the
    /// appropriate aggregate table (`telemetry_hourly` or `telemetry_daily`)
    /// depending on the requested `interval`.
    ///
    /// Returns `(rows, total_count)` where rows are `(timestamp, device_id, metric, value)`.
    pub async fn query_telemetry_export(
        &self,
        device_id: Option<&str>,
        metric: Option<&str>,
        from: Option<i64>,
        to: Option<i64>,
        aggregation: Option<&str>,
        interval: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<(i64, String, String, f64)>, i64)> {
        // Determine the target table, time column, and value column up front
        // so the WHERE clause and count query use the correct names.
        let (table, ts_col, val_col) = match (aggregation, interval) {
            (Some(agg), Some(int)) => {
                let val = match (int, agg) {
                    ("1d", "min") => "min_value",
                    ("1d", "max") => "max_value",
                    ("1d", "count") => "sample_count",
                    ("1d", "sum") => "ROUND(avg_value * sample_count, 4)",
                    ("1d", _) => "avg_value",
                    (_, "min") => "min_value",
                    (_, "max") => "max_value",
                    (_, "count") => "sample_count",
                    (_, "sum") => "ROUND(avg_value * sample_count, 4)",
                    (_, _) => "avg_value",
                };
                let tbl = if int == "1d" { "telemetry_daily" } else { "telemetry_hourly" };
                (tbl, "bucket_start", val)
            }
            _ => ("telemetry", "timestamp", "value"),
        };

        // Determine total count for meta — query the correct table
        let total = if aggregation.is_some() {
            self.count_telemetry_table(table, ts_col, device_id, metric, from, to).await?
        } else {
            self.count_telemetry(device_id, metric, from, to).await?
        };

        let conn = self.conn.clone();
        let device_id = device_id.map(|s| s.to_string());
        let metric = metric.map(|s| s.to_string());
        let table = table.to_string();
        let ts_col = ts_col.to_string();
        let val_col = val_col.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            // Build WHERE clause using the correct time column for the target table
            let mut where_clauses: Vec<String> = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            let mut param_idx = 1usize;

            if let Some(ref d) = device_id {
                where_clauses.push(format!("device_id = ?{param_idx}"));
                params.push(Box::new(d.clone()));
                param_idx += 1;
            }
            if let Some(ref m) = metric {
                where_clauses.push(format!("metric = ?{param_idx}"));
                params.push(Box::new(m.clone()));
                param_idx += 1;
            }
            if let Some(f) = from {
                where_clauses.push(format!("{ts_col} >= ?{param_idx}"));
                params.push(Box::new(f));
                param_idx += 1;
            }
            if let Some(t) = to {
                where_clauses.push(format!("{ts_col} <= ?{param_idx}"));
                params.push(Box::new(t));
                param_idx += 1;
            }

            let where_sql = if where_clauses.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", where_clauses.join(" AND "))
            };

            let sql = format!(
                "SELECT {ts_col}, device_id, metric, {val_col} FROM {table} {where_sql} \
                 ORDER BY {ts_col} DESC LIMIT ?{param_idx} OFFSET ?{param_idx2}",
                ts_col = ts_col,
                val_col = val_col,
                table = table,
                where_sql = where_sql,
                param_idx = param_idx,
                param_idx2 = param_idx + 1,
            );
            params.push(Box::new(limit as i64));
            params.push(Box::new(offset as i64));

            let mut stmt = conn.prepare(&sql).context("Prepare export query failed")?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(param_refs.as_slice(), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, f64>(3)?,
                ))
            }).context("Query export failed")?;
            let rows = rows.collect::<Result<Vec<_>, _>>().context("Row mapping failed")?;

            Ok((rows, total))
        }).await.context("Task join failed")?
    }

    /// Count rows in an arbitrary telemetry table (raw or aggregated) for pagination meta.
    ///
    /// `table` is the table name (e.g. "telemetry", "telemetry_hourly", "telemetry_daily").
    /// `ts_col` is the time column name ("timestamp" for raw, "bucket_start" for aggregated).
    pub async fn count_telemetry_table(
        &self,
        table: &str,
        ts_col: &str,
        device_id: Option<&str>,
        metric: Option<&str>,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let device_id = device_id.map(|s| s.to_string());
        let metric = metric.map(|s| s.to_string());
        let table = table.to_string();
        let ts_col = ts_col.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut sql = format!("SELECT COUNT(*) FROM {table} WHERE 1=1", table = table);
            let mut param_idx = 1usize;

            if let Some(ref _d) = device_id {
                sql.push_str(&format!(" AND device_id = ?{param_idx}"));
                param_idx += 1;
            }
            if let Some(ref _m) = metric {
                sql.push_str(&format!(" AND metric = ?{param_idx}"));
                param_idx += 1;
            }
            if from.is_some() {
                sql.push_str(&format!(" AND {ts_col} >= ?{param_idx}", ts_col = ts_col));
                param_idx += 1;
            }
            if to.is_some() {
                sql.push_str(&format!(" AND {ts_col} <= ?{param_idx}", ts_col = ts_col));
            }

            let mut stmt = conn.prepare(&sql).context("Prepare count failed")?;

            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            if let Some(ref d) = device_id {
                params.push(Box::new(d.clone()));
            }
            if let Some(ref m) = metric {
                params.push(Box::new(m.clone()));
            }
            if let Some(f) = from {
                params.push(Box::new(f));
            }
            if let Some(t) = to {
                params.push(Box::new(t));
            }

            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let count: i64 = stmt.query_row(param_refs.as_slice(), |row| row.get(0))
                .context("Count query failed")?;
            Ok(count)
        }).await.context("Task join failed")?
    }

    // ── Users ──

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let conn = self.conn.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, username, password_hash, salt, COALESCE(role, 'user'), COALESCE(must_change_password, 0), COALESCE(failed_attempts, 0), locked_until FROM users WHERE username = ?1")
                .context("Prepare failed")?;
            let mut rows = stmt
                .query_map(rusqlite::params![username], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password_hash: row.get(2)?,
                        salt: row.get(3)?,
                        role: row.get(4)?,
                        must_change_password: row.get(5)?,
                        failed_attempts: row.get(6)?,
                        locked_until: row.get(7)?,
                    })
                })
                .context("Query failed")?;
            match rows.next() {
                Some(Ok(user)) => Ok(Some(user)),
                Some(Err(e)) => Err(e).context("Row mapping failed"),
                None => Ok(None),
            }
        })
        .await
        .context("Task join failed")?
    }

    pub async fn create_user(&self, username: &str, password_hash: &str, salt: &str, role: &str) -> Result<i64> {
        let conn = self.conn.clone();
        let username = username.to_string();
        let password_hash = password_hash.to_string();
        let salt = salt.to_string();
        let role = role.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO users (username, password_hash, salt, role, failed_attempts, locked_until) VALUES (?1, ?2, ?3, ?4, 0, NULL)",
                rusqlite::params![username, password_hash, salt, role],
            )
            .context("Insert user failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, username, password_hash, salt, COALESCE(role, 'user'), COALESCE(must_change_password, 0), COALESCE(failed_attempts, 0), locked_until FROM users ORDER BY id")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password_hash: row.get(2)?,
                        salt: row.get(3)?,
                        role: row.get(4)?,
                        must_change_password: row.get(5)?,
                        failed_attempts: row.get(6)?,
                        locked_until: row.get(7)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_user_by_id(&self, id: i64) -> Result<Option<User>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, username, password_hash, salt, COALESCE(role, 'user'), COALESCE(must_change_password, 0), COALESCE(failed_attempts, 0), locked_until FROM users WHERE id = ?1")
                .context("Prepare failed")?;
            let result = stmt
                .query_row(rusqlite::params![id], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password_hash: row.get(2)?,
                        salt: row.get(3)?,
                        role: row.get(4)?,
                        must_change_password: row.get(5)?,
                        failed_attempts: row.get(6)?,
                        locked_until: row.get(7)?,
                    })
                })
                .ok();
            Ok(result)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_user_role(&self, id: i64, role: &str) -> Result<()> {
        let conn = self.conn.clone();
        let role = role.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE users SET role = ?1 WHERE id = ?2",
                rusqlite::params![role, id],
            )
            .context("Update user role failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn set_must_change_password(&self, username: &str, flag: bool) -> Result<()> {
        let conn = self.conn.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE users SET must_change_password = ?1 WHERE username = ?2",
                rusqlite::params![flag, username],
            )
            .context("Set must_change_password failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_user_password(&self, id: i64, password_hash: &str) -> Result<()> {
        let conn = self.conn.clone();
        let password_hash = password_hash.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE users SET password_hash = ?1, must_change_password = 0, failed_attempts = 0, locked_until = NULL WHERE id = ?2",
                rusqlite::params![password_hash, id],
            )
            .context("Update user password failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn increment_failed_attempts(&self, username: &str) -> Result<()> {
        let conn = self.conn.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE users SET failed_attempts = COALESCE(failed_attempts, 0) + 1 WHERE username = ?1",
                rusqlite::params![username],
            )
            .context("Increment failed attempts failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn lock_account(&self, username: &str, locked_until: i64) -> Result<()> {
        let conn = self.conn.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE users SET locked_until = ?1 WHERE username = ?2",
                rusqlite::params![locked_until, username],
            )
            .context("Lock account failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn reset_failed_attempts(&self, username: &str) -> Result<()> {
        let conn = self.conn.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE users SET failed_attempts = 0, locked_until = NULL WHERE username = ?1",
                rusqlite::params![username],
            )
            .context("Reset failed attempts failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    // ── Refresh token management ──

    pub async fn create_refresh_token(&self, username: &str, token_hash: &str, expires_at: i64, created_at: i64) -> Result<i64> {
        let conn = self.conn.clone();
        let username = username.to_string();
        let token_hash = token_hash.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO refresh_tokens (token_hash, username, expires_at, created_at) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![token_hash, username, expires_at, created_at],
            )
            .context("Create refresh token failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_refresh_token_by_hash(&self, token_hash: &str) -> Result<Option<crate::models::RefreshToken>> {
        let conn = self.conn.clone();
        let token_hash = token_hash.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, token_hash, username, expires_at, created_at, revoked_at FROM refresh_tokens WHERE token_hash = ?1"
            )
            .context("Prepare failed")?;
            let result = stmt.query_row(
                rusqlite::params![token_hash],
                |row| {
                    Ok(crate::models::RefreshToken {
                        id: row.get(0)?,
                        token_hash: row.get(1)?,
                        username: row.get(2)?,
                        expires_at: row.get(3)?,
                        created_at: row.get(4)?,
                        revoked_at: row.get(5)?,
                    })
                },
            )
            .ok();
            Ok(result)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn revoke_refresh_token(&self, token_hash: &str) -> Result<()> {
        let conn = self.conn.clone();
        let token_hash = token_hash.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE refresh_tokens SET revoked_at = ?1 WHERE token_hash = ?2",
                rusqlite::params![crate::timeutil::now_secs(), token_hash],
            )
            .context("Revoke refresh token failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn revoke_all_user_refresh_tokens(&self, username: &str) -> Result<()> {
        let conn = self.conn.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE refresh_tokens SET revoked_at = ?1 WHERE username = ?2 AND revoked_at IS NULL",
                rusqlite::params![crate::timeutil::now_secs(), username],
            )
            .context("Revoke all user refresh tokens failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_user(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM users WHERE id = ?1", rusqlite::params![id])
                .context("Delete user failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    // ── Devices ──

    pub async fn max_device_sort_order(&self) -> Result<i64> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let max: Option<i64> = conn
                .query_row("SELECT MAX(sort_order) FROM devices", [], |row| row.get(0))
                .ok();
            Ok(max.unwrap_or(-1))
        })
        .await
        .context("Task join failed")?
    }

    pub async fn device_exists(&self, id: &str) -> Result<bool> {
        let conn = self.conn.clone();
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM devices WHERE id = ?1", rusqlite::params![id], |row| row.get(0))
                .context("Check device exists failed")?;
            Ok(count > 0)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, name, type, sort_order FROM devices ORDER BY sort_order")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(Device {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        device_type: row.get(2)?,
                        sort_order: row.get(3)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn create_device(&self, id: &str, name: &str, sort_order: i64) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let name = name.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO devices (id, name, sort_order) VALUES (?1, ?2, ?3)",
                rusqlite::params![id, name, sort_order],
            )
            .context("Insert device failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_device(&self, id: &str, name: Option<&str>, sort_order: Option<i64>) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let name = name.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            if let Some(ref name) = name {
                conn.execute(
                    "UPDATE devices SET name = ?1 WHERE id = ?2",
                    rusqlite::params![name, id],
                )
                .context("Update device name failed")?;
            }
            if let Some(sort_order) = sort_order {
                conn.execute(
                    "UPDATE devices SET sort_order = ?1 WHERE id = ?2",
                    rusqlite::params![sort_order, id],
                )
                .context("Update device sort_order failed")?;
            }
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_device(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM devices WHERE id = ?1", rusqlite::params![id])
                .context("Delete device failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    // ── Data Sources ──

    pub async fn max_data_source_sort_order(&self) -> Result<i64> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let max: Option<i64> = conn
                .query_row("SELECT MAX(sort_order) FROM data_sources", [], |row| row.get(0))
                .ok();
            Ok(max.unwrap_or(-1))
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_data_sources(&self, screen_id: Option<&str>) -> Result<Vec<DataSource>> {
        let conn = self.conn.clone();
        let screen_id = screen_id.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match &screen_id {
                Some(sid) => (
                    "SELECT s.id, s.key, s.name, s.unit, s.color, s.min_val, s.max_val,
                            s.warn_val, s.crit_val, s.base_val, s.sort_order, s.decimals,
                            s.source_type, s.source_config, s.device_id, s.equipment_id, s.chart_type
                     FROM data_sources s
                     JOIN screen_data_sources ss ON s.id = ss.data_source_id
                     WHERE ss.screen_id = ?1
                     ORDER BY ss.sort_order",
                    vec![Box::new(sid.clone())],
                ),
                None => (
                    "SELECT id, key, name, unit, color, min_val, max_val,
                            warn_val, crit_val, base_val, sort_order, decimals,
                            source_type, source_config, device_id, equipment_id, chart_type
                     FROM data_sources
                     ORDER BY sort_order",
                    vec![],
                ),
            };
            let mut stmt = conn.prepare(sql).context("Prepare failed")?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt
                .query_map(param_refs.as_slice(), |row| {
                    Ok(DataSource {
                        id: row.get(0)?,
                        key: row.get(1)?,
                        name: row.get(2)?,
                        unit: row.get(3)?,
                        color: row.get(4)?,
                        min_val: row.get(5)?,
                        max_val: row.get(6)?,
                        warn_val: row.get(7)?,
                        crit_val: row.get(8)?,
                        base_val: row.get(9)?,
                        sort_order: row.get(10)?,
                        decimals: row.get(11)?,
                        source_type: row.get(12)?,
                        source_config: row.get(13)?,
                        device_id: row.get(14)?,
                        equipment_id: row.get(15)?,
                        chart_type: row.get(16)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn create_data_source(
        &self,
        key: &str,
        name: &str,
        unit: &str,
        color: &str,
        min_val: f64,
        max_val: f64,
        warn_val: f64,
        crit_val: f64,
        base_val: f64,
        sort_order: i64,
        decimals: i64,
        source_type: &str,
        source_config: &str,
        device_id: &str,
        equipment_id: Option<&str>,
        chart_type: &str,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let key = key.to_string();
        let name = name.to_string();
        let unit = unit.to_string();
        let color = color.to_string();
        let source_type = source_type.to_string();
        let source_config = source_config.to_string();
        let device_id = device_id.to_string();
        let equipment_id = equipment_id.map(|s| s.to_string());
        let chart_type = chart_type.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO data_sources (key, name, unit, color, min_val, max_val, warn_val, crit_val,
                 base_val, sort_order, decimals, source_type, source_config, device_id, equipment_id, chart_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                rusqlite::params![key, name, unit, color, min_val, max_val, warn_val, crit_val,
                                  base_val, sort_order, decimals, source_type, source_config,
                                  device_id, equipment_id, chart_type],
            )
            .context("Insert data source failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_data_source(&self, id: i64, fields: &serde_json::Value) -> Result<()> {
        let conn = self.conn.clone();
        let fields = fields.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let obj = fields
                .as_object()
                .context("fields must be a JSON object")?;
            let allowed = [
                "key", "name", "unit", "color", "min_val", "max_val",
                "warn_val", "crit_val", "base_val", "sort_order", "decimals",
                "source_type", "source_config", "device_id", "equipment_id", "chart_type",
            ];
            let mut parts = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            for &col in &allowed {
                if let Some(val) = obj.get(col) {
                    parts.push(format!("{col} = ?"));
                    match val {
                        serde_json::Value::Number(n) => {
                            if let Some(f) = n.as_f64() {
                                params.push(Box::new(f));
                            } else if let Some(i) = n.as_i64() {
                                params.push(Box::new(i));
                            }
                        }
                        serde_json::Value::String(s) => {
                            params.push(Box::new(s.clone()));
                        }
                        serde_json::Value::Null => {
                            params.push(Box::new(Option::<String>::None));
                        }
                        other => {
                            params.push(Box::new(other.to_string()));
                        }
                    }
                }
            }
            if parts.is_empty() {
                return Ok(());
            }
            params.push(Box::new(id));
            let sql = format!("UPDATE data_sources SET {} WHERE id = ?", parts.join(", "));
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params.iter().map(|p| p.as_ref()).collect();
            conn.execute(&sql, param_refs.as_slice())
                .context("Update data source failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_data_source(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM data_sources WHERE id = ?1", rusqlite::params![id])
                .context("Delete data source failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn reorder_data_sources(&self, order: &[i64]) -> Result<()> {
        let conn = self.conn.clone();
        let order = order.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            for (i, &id) in order.iter().enumerate() {
                conn.execute(
                    "UPDATE data_sources SET sort_order = ?1 WHERE id = ?2",
                    rusqlite::params![i as i64, id],
                )
                .context("Reorder data source failed")?;
            }
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    // ── Screen Data Sources ──

    pub async fn add_screen_data_source(
        &self,
        screen_id: &str,
        data_source_id: i64,
        sort_order: i64,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO screen_data_sources (screen_id, data_source_id, sort_order) VALUES (?1, ?2, ?3)",
                rusqlite::params![screen_id, data_source_id, sort_order],
            )
            .context("Add screen data source failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_screen_data_source_sort(
        &self,
        screen_id: &str,
        data_source_id: i64,
        sort_order: i64,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE screen_data_sources SET sort_order = ?1 WHERE screen_id = ?2 AND data_source_id = ?3",
                rusqlite::params![sort_order, screen_id, data_source_id],
            )
            .context("Update screen data source sort failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn remove_screen_data_source(&self, screen_id: &str, data_source_id: i64) -> Result<()> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "DELETE FROM screen_data_sources WHERE screen_id = ?1 AND data_source_id = ?2",
                rusqlite::params![screen_id, data_source_id],
            )
            .context("Remove screen data source failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_screen_data_sources_by_screen(&self, screen_id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "DELETE FROM screen_data_sources WHERE screen_id = ?1",
                rusqlite::params![screen_id],
            )
            .context("Delete screen data sources by screen failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    // ── Screen Layouts ──

    pub async fn get_layouts(&self, screen_id: &str) -> Result<Vec<ScreenLayoutRow>> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT screen_id, breakpoint, layout FROM screen_layouts WHERE screen_id = ?1")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map(rusqlite::params![screen_id], |row| {
                    Ok(ScreenLayoutRow {
                        screen_id: row.get(0)?,
                        breakpoint: row.get(1)?,
                        layout: row.get(2)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn save_layouts(
        &self,
        screen_id: &str,
        layouts: &serde_json::Value,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        let layouts = layouts.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            tx.execute(
                "DELETE FROM screen_layouts WHERE screen_id = ?1",
                rusqlite::params![screen_id],
            )
            .context("Delete layouts failed")?;
            if let Some(arr) = layouts.as_array() {
                for item in arr {
                    let bp = item.get("breakpoint")
                        .and_then(|v| v.as_str())
                        .unwrap_or("lg");
                    let layout = item.get("layout")
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "[]".to_string());
                    tx.execute(
                        "INSERT INTO screen_layouts (screen_id, breakpoint, layout) VALUES (?1, ?2, ?3)",
                        rusqlite::params![screen_id, bp, layout],
                    )
                    .context("Insert layout failed")?;
                }
            }
            tx.commit().context("Commit failed")
        })
        .await
        .context("Task join failed")?
    }

    // ── Twin Layouts ──

    pub async fn get_twin_layouts(&self, screen_id: &str) -> Result<Vec<TwinLayout>> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            // Try with equipment_name column first, fall back to without
            let mut stmt = conn
                .prepare("SELECT screen_id, equipment_id, equipment_type, COALESCE(equipment_name, ''), position_x, position_z, rotation_y FROM twin_layouts WHERE screen_id = ?1")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map(rusqlite::params![screen_id], |row| {
                    Ok(TwinLayout {
                        screen_id: row.get(0)?,
                        equipment_id: row.get(1)?,
                        equipment_type: row.get(2)?,
                        equipment_name: row.get(3)?,
                        position_x: row.get(4)?,
                        position_z: row.get(5)?,
                        rotation_y: row.get(6)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn save_twin_layouts(&self, screen_id: &str, layouts: &[TwinLayout]) -> Result<()> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        let layouts = layouts.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            tx.execute(
                "DELETE FROM twin_layouts WHERE screen_id = ?1",
                rusqlite::params![screen_id],
            )
            .context("Delete twin layouts failed")?;
            for l in &layouts {
                tx.execute(
                    "INSERT INTO twin_layouts (screen_id, equipment_id, equipment_type, equipment_name, position_x, position_z, rotation_y)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![l.screen_id, l.equipment_id, l.equipment_type, l.equipment_name,
                                      l.position_x, l.position_z, l.rotation_y],
                )
                .context("Insert twin layout failed")?;
            }
            tx.commit().context("Commit failed")
        })
        .await
        .context("Task join failed")?
    }

    // ── Twin Connections ──

    pub async fn get_twin_connections(&self, screen_id: &str) -> Result<Vec<TwinConnection>> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT screen_id, from_id, to_id FROM twin_connections WHERE screen_id = ?1")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map(rusqlite::params![screen_id], |row| {
                    Ok(TwinConnection {
                        screen_id: row.get(0)?,
                        from_id: row.get(1)?,
                        to_id: row.get(2)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn save_twin_connections(&self, screen_id: &str, connections: &[TwinConnection]) -> Result<()> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        let connections = connections.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            tx.execute(
                "DELETE FROM twin_connections WHERE screen_id = ?1",
                rusqlite::params![screen_id],
            )
            .context("Delete twin connections failed")?;
            for c in &connections {
                tx.execute(
                    "INSERT INTO twin_connections (screen_id, from_id, to_id)
                     VALUES (?1, ?2, ?3)",
                    rusqlite::params![c.screen_id, c.from_id, c.to_id],
                )
                .context("Insert twin connection failed")?;
            }
            tx.commit().context("Commit failed")
        })
        .await
        .context("Task join failed")?
    }

    // ── Canvas Layouts ──

    pub async fn get_canvas_layouts(&self, screen_id: &str) -> Result<Vec<CanvasLayoutItem>> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT screen_id, data_source_id, x, y, width, height FROM canvas_layouts WHERE screen_id = ?1")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map(rusqlite::params![screen_id], |row| {
                    Ok(CanvasLayoutItem {
                        screen_id: row.get(0)?,
                        data_source_id: row.get(1)?,
                        x: row.get(2)?,
                        y: row.get(3)?,
                        width: row.get(4)?,
                        height: row.get(5)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn save_canvas_layouts(&self, screen_id: &str, layouts: &[CanvasLayoutItem]) -> Result<()> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        let layouts = layouts.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            tx.execute(
                "DELETE FROM canvas_layouts WHERE screen_id = ?1",
                rusqlite::params![screen_id],
            )
            .context("Delete canvas layouts failed")?;
            for l in &layouts {
                tx.execute(
                    "INSERT INTO canvas_layouts (screen_id, data_source_id, x, y, width, height)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![l.screen_id, l.data_source_id, l.x, l.y, l.width, l.height],
                )
                .context("Insert canvas layout failed")?;
            }
            tx.commit().context("Commit failed")
        })
        .await
        .context("Task join failed")?
    }

    // ── SCADA 2D Equipment ──

    pub async fn get_scada_equipment(&self, screen_id: &str) -> Result<Vec<ScadaEquipment>> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT screen_id, equipment_id, equipment_type, equipment_name, x, y, width, height, rotation, sensor_bindings FROM scada_equipment WHERE screen_id = ?1")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map(rusqlite::params![screen_id], |row| {
                    Ok(ScadaEquipment {
                        screen_id: row.get(0)?,
                        equipment_id: row.get(1)?,
                        equipment_type: row.get(2)?,
                        equipment_name: row.get(3)?,
                        x: row.get(4)?,
                        y: row.get(5)?,
                        width: row.get(6)?,
                        height: row.get(7)?,
                        rotation: row.get(8)?,
                        sensor_bindings: row.get(9)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn save_scada_equipment(&self, screen_id: &str, items: &[ScadaEquipment]) -> Result<()> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        let items = items.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            tx.execute(
                "DELETE FROM scada_equipment WHERE screen_id = ?1",
                rusqlite::params![screen_id],
            )
            .context("Delete scada equipment failed")?;
            for item in &items {
                tx.execute(
                    "INSERT INTO scada_equipment (screen_id, equipment_id, equipment_type, equipment_name, x, y, width, height, rotation, sensor_bindings)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    rusqlite::params![item.screen_id, item.equipment_id, item.equipment_type, item.equipment_name,
                                      item.x, item.y, item.width, item.height, item.rotation, item.sensor_bindings],
                )
                .context("Insert scada equipment failed")?;
            }
            tx.commit().context("Commit failed")
        })
        .await
        .context("Task join failed")?
    }

    // ── SCADA 2D Connections ──

    pub async fn get_scada_connections(&self, screen_id: &str) -> Result<Vec<ScadaConnection>> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT screen_id, connection_id, from_equipment, to_equipment, waypoints, connection_type FROM scada_connections WHERE screen_id = ?1")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map(rusqlite::params![screen_id], |row| {
                    Ok(ScadaConnection {
                        screen_id: row.get(0)?,
                        connection_id: row.get(1)?,
                        from_equipment: row.get(2)?,
                        to_equipment: row.get(3)?,
                        waypoints: row.get(4)?,
                        connection_type: row.get(5)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn save_scada_connections(&self, screen_id: &str, connections: &[ScadaConnection]) -> Result<()> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        let connections = connections.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            tx.execute(
                "DELETE FROM scada_connections WHERE screen_id = ?1",
                rusqlite::params![screen_id],
            )
            .context("Delete scada connections failed")?;
            for c in &connections {
                tx.execute(
                    "INSERT INTO scada_connections (screen_id, connection_id, from_equipment, to_equipment, waypoints, connection_type)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![c.screen_id, c.connection_id, c.from_equipment, c.to_equipment,
                                      c.waypoints, c.connection_type],
                )
                .context("Insert scada connection failed")?;
            }
            tx.commit().context("Commit failed")
        })
        .await
        .context("Task join failed")?
    }

    // ── Settings ──

    pub async fn get_settings(&self, keys: &[&str]) -> Result<Vec<Setting>> {
        let conn = self.conn.clone();
        let keys: Vec<String> = keys.iter().map(|s| s.to_string()).collect();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let placeholders: Vec<String> = keys.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
            let sql = format!(
                "SELECT key, value FROM settings WHERE key IN ({})",
                placeholders.join(", ")
            );
            let mut stmt = conn.prepare(&sql).context("Prepare failed")?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                keys.iter().map(|k| k as &dyn rusqlite::types::ToSql).collect();
            let rows = stmt
                .query_map(param_refs.as_slice(), |row| {
                    Ok(Setting {
                        key: row.get(0)?,
                        value: row.get(1)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn upsert_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.clone();
        let key = key.to_string();
        let value = value.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                rusqlite::params![key, value],
            )
            .context("Upsert setting failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    // ── Audit Log ──

    pub async fn log_audit_event(
        &self,
        action: &str,
        username: &str,
        resource: &str,
        details: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let action = action.to_string();
        let username = username.to_string();
        let resource = resource.to_string();
        let details = details.map(|s| s.to_string());
        let now = crate::timeutil::now_secs();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO audit_log (action, username, resource, details, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![action, username, resource, details, now],
            )
            .context("Insert audit log failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_audit_logs(&self, limit: i64) -> Result<Vec<(i64, String, String, String, Option<String>, i64)>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, action, username, resource, details, created_at
                 FROM audit_log
                 ORDER BY created_at DESC
                 LIMIT ?1"
            ).context("Prepare failed")?;
            let rows = stmt.query_map(rusqlite::params![limit], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            }).context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>().context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    // ── Shortcuts ──

    pub async fn get_user_shortcuts(&self, user_id: i64) -> Result<Vec<UserShortcut>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT user_id, data_source_id, shortcut FROM user_shortcuts WHERE user_id = ?1")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map(rusqlite::params![user_id], |row| {
                    Ok(UserShortcut {
                        user_id: row.get(0)?,
                        data_source_id: row.get(1)?,
                        shortcut: row.get(2)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn save_user_shortcuts(
        &self,
        user_id: i64,
        entries: &[(i64, Option<&str>)],
    ) -> Result<()> {
        let conn = self.conn.clone();
        let entries: Vec<(i64, Option<String>)> = entries
            .iter()
            .map(|(dsid, sc)| (*dsid, sc.map(|s| s.to_string())))
            .collect();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            // Ensure table exists (guard against DBs created before this feature)
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS user_shortcuts (
                    user_id INTEGER NOT NULL,
                    data_source_id INTEGER NOT NULL,
                    shortcut TEXT NOT NULL,
                    PRIMARY KEY (user_id, data_source_id)
                )",
            )
            .context("Ensure user_shortcuts table failed")?;
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            tx.execute(
                "DELETE FROM user_shortcuts WHERE user_id = ?1",
                rusqlite::params![user_id],
            )
            .context("Delete shortcuts failed")?;
            for (data_source_id, shortcut) in &entries {
                let sc = shortcut.as_deref().unwrap_or("");
                tx.execute(
                    "INSERT INTO user_shortcuts (user_id, data_source_id, shortcut) VALUES (?1, ?2, ?3)",
                    rusqlite::params![user_id, data_source_id, sc],
                )
                .with_context(|| format!("Insert shortcut failed: user_id={user_id}, data_source_id={data_source_id}, shortcut={sc:?}"))?;
            }
            tx.commit().context("Commit failed")
        })
        .await
        .context("Task join failed")?
    }

    // ── Nav Shortcuts ──

    pub async fn get_user_nav_shortcuts(&self, user_id: i64) -> Result<Vec<NavShortcut>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS user_nav_shortcuts (
                    user_id INTEGER NOT NULL,
                    target TEXT NOT NULL,
                    shortcut TEXT NOT NULL,
                    PRIMARY KEY (user_id, target)
                )",
            )
            .context("Ensure user_nav_shortcuts table failed")?;
            let mut stmt = conn
                .prepare("SELECT user_id, target, shortcut FROM user_nav_shortcuts WHERE user_id = ?1")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map(rusqlite::params![user_id], |row| {
                    Ok(NavShortcut {
                        user_id: row.get(0)?,
                        target: row.get(1)?,
                        shortcut: row.get(2)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn save_user_nav_shortcuts(
        &self,
        user_id: i64,
        entries: &[(String, Option<&str>)],
    ) -> Result<()> {
        let conn = self.conn.clone();
        let entries: Vec<(String, Option<String>)> = entries
            .iter()
            .map(|(t, sc)| (t.clone(), sc.map(|s| s.to_string())))
            .collect();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS user_nav_shortcuts (
                    user_id INTEGER NOT NULL,
                    target TEXT NOT NULL,
                    shortcut TEXT NOT NULL,
                    PRIMARY KEY (user_id, target)
                )",
            )
            .context("Ensure user_nav_shortcuts table failed")?;
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            tx.execute(
                "DELETE FROM user_nav_shortcuts WHERE user_id = ?1",
                rusqlite::params![user_id],
            )
            .context("Delete nav shortcuts failed")?;
            for (target, shortcut) in &entries {
                let sc = shortcut.as_deref().unwrap_or("");
                tx.execute(
                    "INSERT INTO user_nav_shortcuts (user_id, target, shortcut) VALUES (?1, ?2, ?3)",
                    rusqlite::params![user_id, target, sc],
                )
                .with_context(|| format!("Insert nav shortcut failed: user_id={user_id}, target={target}, shortcut={sc:?}"))?;
            }
            tx.commit().context("Commit failed")
        })
        .await
        .context("Task join failed")?
    }

    // ── Seed ──

    pub async fn seed_database(&self) -> Result<()> {
        let existing = self.get_user_by_username("admin").await?;
        if existing.is_none() {
            use rand::Rng;
            let password: String = rand::rng()
                .sample_iter(&rand::distr::Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();
            let password_hash = crate::api::auth::hash_password(&password);

            self.create_user("admin", &password_hash, "", "admin").await?;
            self.set_must_change_password("admin", true).await?;

            tracing::warn!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            tracing::warn!("  GENERATED ADMIN PASSWORD: {}", password);
            tracing::warn!("  You will be required to change this on first login.");
            tracing::warn!("  Save this password now — it will NOT be shown again.");
            tracing::warn!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }

        // Seed PLTS data sources if none exist
        let all_data_sources = self.list_data_sources(None).await?;
        if all_data_sources.is_empty() {
            self.seed_plts_data_sources().await?;
        }

        Ok(())
    }

    async fn seed_plts_data_sources(&self) -> Result<()> {
        // Seed devices
        if !self.device_exists("plts_1").await? {
            self.create_device("plts_1", "PLTS 1", 0).await?;
        }
        if !self.device_exists("plts_2").await? {
            self.create_device("plts_2", "PLTS 2", 1).await?;
        }

        // Data source definitions matching Modbus register names from config.yaml
        // (key, name, unit, color, min, max, warn, crit, base, sort, decimals, source_type, source_config)
        let data_source_defs: &[(&str, &str, &str, &str, f64, f64, f64, f64, f64, i64, i64, &str, &str)] = &[
            ("voltage", "Voltage", "V", "#f59e0b", 0.0, 450.0, 400.0, 420.0, 380.0, 0, 1, "modbus", "{}"),
            ("current", "Current", "A", "#06b6d4", 0.0, 100.0, 85.0, 95.0, 65.0, 1, 2, "modbus", "{}"),
            ("power", "Power", "W", "#a855f7", 0.0, 50000.0, 40000.0, 45000.0, 24700.0, 2, 0, "modbus", "{}"),
            ("frequency", "Frequency", "Hz", "#22c55e", 45.0, 55.0, 51.0, 52.0, 50.0, 3, 2, "modbus", "{}"),
            ("power_factor", "Power Factor", "", "#ef4444", 0.0, 1.0, 0.95, 0.98, 0.92, 4, 3, "modbus", "{}"),
            ("energy_total", "Energy Total", "kWh", "#8b5cf6", 0.0, 999999.0, 800000.0, 900000.0, 158732.0, 5, 1, "modbus", "{}"),
        ];

        let mut plts1_data_source_ids: Vec<i64> = Vec::new();
        let mut plts2_data_source_ids: Vec<i64> = Vec::new();

        for (i, def) in data_source_defs.iter().enumerate() {
            let plts1_id = self.create_data_source(
                def.0, def.1, def.2, def.3, def.4, def.5, def.6, def.7,
                def.8, def.9, def.10, def.11, def.12, "plts_1", None, "sparkline",
            ).await?;
            plts1_data_source_ids.push(plts1_id);

            let plts2_id = self.create_data_source(
                def.0, def.1, def.2, def.3, def.4, def.5, def.6, def.7,
                def.8, i as i64 + 6, def.10, def.11, def.12, "plts_2", None, "sparkline",
            ).await?;
            plts2_data_source_ids.push(plts2_id);
        }

        // Assign data sources to screen "plts_1"
        for (i, &sid) in plts1_data_source_ids.iter().enumerate() {
            self.add_screen_data_source("plts_1", sid, i as i64).await?;
        }

        // Assign data sources to screen "plts_2"
        for (i, &sid) in plts2_data_source_ids.iter().enumerate() {
            self.add_screen_data_source("plts_2", sid, i as i64).await?;
        }

        tracing::info!("Seeded PLTS data sources for plts_1 and plts_2");
        Ok(())
    }

    // ── Alarm methods ──

    pub async fn create_alarm(
        &self,
        source_key: &str,
        severity: &str,
        value: f64,
        threshold: f64,
        message: &str,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let source_key = source_key.to_string();
        let severity = severity.to_string();
        let message = message.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO alarms (source_key, severity, value, threshold, message, triggered_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![source_key, severity, value, threshold, message, now],
            )
            .context("Insert alarm failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn resolve_active_alarms(&self, source_key: &str) -> Result<()> {
        let conn = self.conn.clone();
        let source_key = source_key.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "UPDATE alarms SET resolved_at = ?1
                 WHERE source_key = ?2 AND resolved_at IS NULL AND acknowledged_at IS NOT NULL",
                rusqlite::params![now, source_key],
            )
            .context("Resolve alarms failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }

    pub async fn count_active_alarms(&self) -> Result<i64> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM alarms
                     WHERE acknowledged_at IS NULL AND resolved_at IS NULL",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            Ok(count)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn has_active_alarm(&self, source_key: &str, severity: &str) -> Result<bool> {
        let conn = self.conn.clone();
        let source_key = source_key.to_string();
        let severity = severity.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM alarms
                     WHERE source_key = ?1 AND severity = ?2
                       AND acknowledged_at IS NULL AND resolved_at IS NULL",
                    rusqlite::params![source_key, severity],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            Ok(count > 0)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_alarms(&self, status: Option<&str>, limit: i64) -> Result<Vec<Alarm>> {
        let conn = self.conn.clone();
        let status = status.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match &status {
                Some(s) if s == "active" => (
                    "SELECT id, source_key, severity, value, threshold, message,
                            triggered_at, resolved_at, acknowledged_at, acknowledged_by, cooldown_until
                     FROM alarms
                     WHERE acknowledged_at IS NULL
                     ORDER BY triggered_at DESC LIMIT ?1",
                    vec![Box::new(limit)],
                ),
                Some(s) if s == "acknowledged" => (
                    "SELECT id, source_key, severity, value, threshold, message,
                            triggered_at, resolved_at, acknowledged_at, acknowledged_by, cooldown_until
                     FROM alarms
                     WHERE acknowledged_at IS NOT NULL
                     ORDER BY triggered_at DESC LIMIT ?1",
                    vec![Box::new(limit)],
                ),
                _ => (
                    "SELECT id, source_key, severity, value, threshold, message,
                            triggered_at, resolved_at, acknowledged_at, acknowledged_by, cooldown_until
                     FROM alarms
                     ORDER BY triggered_at DESC LIMIT ?1",
                    vec![Box::new(limit)],
                ),
            };
            let mut stmt = conn.prepare(sql).context("Prepare failed")?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt
                .query_map(param_refs.as_slice(), |row| {
                    Ok(Alarm {
                        id: row.get(0)?,
                        source_key: row.get(1)?,
                        source_name: String::new(),
                        severity: row.get(2)?,
                        value: row.get(3)?,
                        threshold: row.get(4)?,
                        message: row.get(5)?,
                        triggered_at: row.get(6)?,
                        resolved_at: row.get(7)?,
                        acknowledged_at: row.get(8)?,
                        acknowledged_by: row.get(9)?,
                        cooldown_until: row.get(10)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn acknowledge_alarm(&self, id: i64, username: &str, cooldown_secs: i64) -> Result<()> {
        let conn = self.conn.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            let cooldown_until = if cooldown_secs > 0 { Some(now + cooldown_secs) } else { None };
            conn.execute(
                "UPDATE alarms SET acknowledged_at = ?1, acknowledged_by = ?2, cooldown_until = ?3 WHERE id = ?4",
                rusqlite::params![now, username, cooldown_until, id],
            )
            .context("Acknowledge alarm failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }

    pub async fn acknowledge_all(&self, username: &str, cooldown_secs: i64) -> Result<()> {
        let conn = self.conn.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            let cooldown_until = if cooldown_secs > 0 { Some(now + cooldown_secs) } else { None };
            conn.execute(
                "UPDATE alarms SET acknowledged_at = ?1, acknowledged_by = ?2, cooldown_until = ?3
                 WHERE acknowledged_at IS NULL",
                rusqlite::params![now, username, cooldown_until],
            )
            .context("Acknowledge all alarms failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }

    // ── Notification Settings ──

    pub async fn get_notification_settings_by_source(&self, source_key: &str) -> Result<Vec<crate::models::NotificationSetting>> {
        let conn = self.conn.clone();
        let source_key = source_key.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, source_key, channel_type, config, enabled, created_at
                 FROM notification_settings
                 WHERE source_key = ?1",
            )?;
            let rows = stmt.query_map(rusqlite::params![source_key], |row| {
                Ok(crate::models::NotificationSetting {
                    id: row.get(0)?,
                    source_key: row.get(1)?,
                    channel_type: row.get(2)?,
                    config: row.get(3)?,
                    enabled: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                })
            })?;
            let mut settings = Vec::new();
            for row in rows {
                settings.push(row?);
            }
            Ok(settings)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_all_notification_settings(&self) -> Result<Vec<crate::models::NotificationSetting>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, source_key, channel_type, config, enabled, created_at
                 FROM notification_settings
                 ORDER BY id",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(crate::models::NotificationSetting {
                    id: row.get(0)?,
                    source_key: row.get(1)?,
                    channel_type: row.get(2)?,
                    config: row.get(3)?,
                    enabled: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                })
            })?;
            let mut settings = Vec::new();
            for row in rows {
                settings.push(row?);
            }
            Ok(settings)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn create_notification_setting(
        &self,
        source_key: &str,
        channel_type: &str,
        config: &str,
        enabled: bool,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let source_key = source_key.to_string();
        let channel_type = channel_type.to_string();
        let config = config.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO notification_settings (source_key, channel_type, config, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![source_key, channel_type, config, enabled as i32, now],
            )
            .context("Insert notification setting failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_notification_setting(
        &self,
        id: i64,
        source_key: Option<&str>,
        channel_type: Option<&str>,
        config: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let source_key = source_key.map(|s| s.to_string());
        let channel_type = channel_type.map(|s| s.to_string());
        let config = config.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            if let Some(ref sk) = source_key {
                conn.execute(
                    "UPDATE notification_settings SET source_key = ?1 WHERE id = ?2",
                    rusqlite::params![sk, id],
                )?;
            }
            if let Some(ref ct) = channel_type {
                conn.execute(
                    "UPDATE notification_settings SET channel_type = ?1 WHERE id = ?2",
                    rusqlite::params![ct, id],
                )?;
            }
            if let Some(ref c) = config {
                conn.execute(
                    "UPDATE notification_settings SET config = ?1 WHERE id = ?2",
                    rusqlite::params![c, id],
                )?;
            }
            if let Some(e) = enabled {
                conn.execute(
                    "UPDATE notification_settings SET enabled = ?1 WHERE id = ?2",
                    rusqlite::params![e as i32, id],
                )?;
            }
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_notification_setting(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "DELETE FROM notification_settings WHERE id = ?1",
                rusqlite::params![id],
            )
            .context("Delete notification setting failed")
            .map(|_| ())
        })
        .await
        .context("Task join failed")?
    }

    // ── Flows ──

    pub async fn create_flow(
        &self,
        id: &str,
        name: &str,
        enabled: bool,
        trigger_config: &str,
        nodes: &str,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let name = name.to_string();
        let trigger_config = trigger_config.to_string();
        let nodes = nodes.to_string();
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO flows (id, name, enabled, trigger_config, nodes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![id, name, enabled as i64, trigger_config, nodes, now, now],
            )
            .context("Insert flow failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_flow(&self, id: &str) -> Result<Option<Flow>> {
        let conn = self.conn.clone();
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, name, enabled, trigger_config, nodes, created_at, updated_at FROM flows WHERE id = ?1")
                .context("Prepare failed")?;
            let result = stmt
                .query_row(rusqlite::params![id], |row| {
                    Ok(Flow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        enabled: row.get::<_, i64>(2)? != 0,
                        trigger_config: row.get(3)?,
                        nodes: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                })
                .ok();
            Ok(result)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_flows(&self) -> Result<Vec<Flow>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, name, enabled, trigger_config, nodes, created_at, updated_at FROM flows ORDER BY created_at")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(Flow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        enabled: row.get::<_, i64>(2)? != 0,
                        trigger_config: row.get(3)?,
                        nodes: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_flow(
        &self,
        id: &str,
        name: &str,
        enabled: bool,
        trigger_config: &str,
        nodes: &str,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let name = name.to_string();
        let trigger_config = trigger_config.to_string();
        let nodes = nodes.to_string();
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE flows SET name = ?1, enabled = ?2, trigger_config = ?3, nodes = ?4, updated_at = ?5 WHERE id = ?6",
                rusqlite::params![name, enabled as i64, trigger_config, nodes, now, id],
            )
            .context("Update flow failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_flow(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM flows WHERE id = ?1", rusqlite::params![id])
                .context("Delete flow failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    // ── Flow runs ──

    pub async fn insert_flow_run(
        &self,
        flow_id: &str,
        duration_ms: u64,
        success: bool,
        error: Option<String>,
        output: Option<serde_json::Value>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let flow_id = flow_id.to_string();
        let error = error.map(|s| s.to_string());
        let output = output.map(|v| v.to_string());
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO flow_runs (flow_id, started_at, duration_ms, success, error, output)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![flow_id, now, duration_ms as i64, success as i64, error, output],
            )
            .context("Insert flow run failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_flow_runs(&self, flow_id: &str, limit: i64) -> Result<Vec<FlowRun>> {
        let conn = self.conn.clone();
        let flow_id = flow_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, flow_id, started_at, duration_ms, success, error, output FROM flow_runs WHERE flow_id = ?1 ORDER BY started_at DESC LIMIT ?2")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map(rusqlite::params![flow_id, limit], |row| {
                    Ok(FlowRun {
                        id: row.get(0)?,
                        flow_id: row.get(1)?,
                        started_at: row.get(2)?,
                        duration_ms: row.get(3)?,
                        success: row.get::<_, i64>(4)? != 0,
                        error: row.get(5)?,
                        output: row.get(6)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn is_in_cooldown(&self, source_key: &str, severity: &str) -> Result<bool> {
        let conn = self.conn.clone();
        let source_key = source_key.to_string();
        let severity = severity.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM alarms
                     WHERE source_key = ?1 AND severity = ?2
                       AND cooldown_until IS NOT NULL AND cooldown_until > ?3",
                    rusqlite::params![source_key, severity, now],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            Ok(count > 0)
        })
        .await
        .context("Task join failed")?
    }

    // ── Push subscription stubs ──

    pub async fn save_push_subscription(
        &self,
        user: &str,
        endpoint: &str,
        p256dh: &str,
        auth_key: &str,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let user = user.to_string();
        let endpoint = endpoint.to_string();
        let p256dh = p256dh.to_string();
        let auth_key = auth_key.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO push_subscriptions (user, endpoint, p256dh, auth_key, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(user, endpoint) DO UPDATE SET p256dh=excluded.p256dh, auth_key=excluded.auth_key",
                rusqlite::params![user, endpoint, p256dh, auth_key, now],
            )
            .context("Upsert push subscription failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }

    pub async fn remove_push_subscription(&self, user: &str, endpoint: &str) -> Result<()> {
        let conn = self.conn.clone();
        let user = user.to_string();
        let endpoint = endpoint.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "DELETE FROM push_subscriptions WHERE user = ?1 AND endpoint = ?2",
                rusqlite::params![user, endpoint],
            )
            .context("Delete push subscription failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }

    /// Get all push subscriptions (for broadcasting alarm notifications).
    pub async fn get_push_subscriptions(&self) -> Result<Vec<crate::models::PushSubscription>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, user, endpoint, p256dh, auth_key, created_at
                     FROM push_subscriptions",
                )
                .context("Prepare get_push_subscriptions failed")?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(crate::models::PushSubscription {
                        id: row.get(0)?,
                        user: row.get(1)?,
                        endpoint: row.get(2)?,
                        p256dh: row.get(3)?,
                        auth_key: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                })
                .context("Query get_push_subscriptions failed")?;

            let mut subscriptions = Vec::new();
            for row in rows {
                subscriptions.push(row.context("Row get_push_subscriptions failed")?);
            }
            Ok(subscriptions)
        })
        .await
        .context("Task join failed")?
    }

    /// Get push subscriptions for a specific user.
    pub async fn get_push_subscriptions_by_user(&self, user: &str) -> Result<Vec<crate::models::PushSubscription>> {
        let conn = self.conn.clone();
        let user = user.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, user, endpoint, p256dh, auth_key, created_at
                     FROM push_subscriptions WHERE user = ?1",
                )
                .context("Prepare get_push_subscriptions_by_user failed")?;

            let rows = stmt
                .query_map(rusqlite::params![user], |row| {
                    Ok(crate::models::PushSubscription {
                        id: row.get(0)?,
                        user: row.get(1)?,
                        endpoint: row.get(2)?,
                        p256dh: row.get(3)?,
                        auth_key: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                })
                .context("Query get_push_subscriptions_by_user failed")?;

            let mut subscriptions = Vec::new();
            for row in rows {
                subscriptions.push(row.context("Row get_push_subscriptions_by_user failed")?);
            }
            Ok(subscriptions)
        })
        .await
        .context("Task join failed")?
    }

    /// Delete a push subscription by ID (for cleaning up stale subscriptions).
    pub async fn delete_push_subscription(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "DELETE FROM push_subscriptions WHERE id = ?1",
                rusqlite::params![id],
            )
            .context("Delete push subscription by id failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS telemetry (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            metric TEXT NOT NULL,
            value REAL NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_device_time_metric
            ON telemetry(device_id, metric, timestamp);
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            salt TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user'
        );
        CREATE TABLE IF NOT EXISTS devices (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            type TEXT NOT NULL DEFAULT 'screen',
            sort_order INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS data_sources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key TEXT NOT NULL,
            name TEXT NOT NULL,
            unit TEXT NOT NULL DEFAULT '',
            color TEXT NOT NULL DEFAULT '#f97316',
            min_val REAL NOT NULL,
            max_val REAL NOT NULL,
            warn_val REAL NOT NULL,
            crit_val REAL NOT NULL,
            base_val REAL NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            decimals INTEGER NOT NULL DEFAULT 1,
            source_type TEXT NOT NULL DEFAULT 'random_walk',
            source_config TEXT NOT NULL DEFAULT '{}',
            device_id TEXT NOT NULL DEFAULT 'default',
            equipment_id TEXT
        );
        CREATE TABLE IF NOT EXISTS screen_data_sources (
            screen_id TEXT NOT NULL,
            data_source_id INTEGER NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (screen_id, data_source_id)
        );
        CREATE TABLE IF NOT EXISTS screen_layouts (
            screen_id TEXT NOT NULL,
            breakpoint TEXT NOT NULL,
            layout TEXT NOT NULL DEFAULT '[]',
            PRIMARY KEY (screen_id, breakpoint)
        );
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS user_shortcuts (
            user_id INTEGER NOT NULL,
            data_source_id INTEGER NOT NULL,
            shortcut TEXT NOT NULL,
            PRIMARY KEY (user_id, data_source_id)
        );
        CREATE TABLE IF NOT EXISTS twin_layouts (
            screen_id TEXT NOT NULL,
            equipment_id TEXT NOT NULL,
            equipment_type TEXT NOT NULL,
            position_x REAL NOT NULL DEFAULT 0,
            position_z REAL NOT NULL DEFAULT 0,
            rotation_y REAL NOT NULL DEFAULT 0,
            PRIMARY KEY (screen_id, equipment_id)
        );
        CREATE TABLE IF NOT EXISTS alarms (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_key TEXT NOT NULL,
            severity TEXT NOT NULL CHECK(severity IN ('warning', 'critical')),
            value REAL NOT NULL,
            threshold REAL NOT NULL,
            message TEXT NOT NULL,
            triggered_at INTEGER NOT NULL,
            resolved_at INTEGER,
            acknowledged_at INTEGER,
            acknowledged_by TEXT,
            cooldown_until INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_alarms_active
            ON alarms(source_key, severity, acknowledged_at);
        CREATE INDEX IF NOT EXISTS idx_alarms_triggered
            ON alarms(triggered_at DESC);
        CREATE TABLE IF NOT EXISTS push_subscriptions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            p256dh TEXT NOT NULL,
            auth_key TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            UNIQUE(user, endpoint)
        );
        CREATE TABLE IF NOT EXISTS canvas_layouts (
            screen_id TEXT NOT NULL,
            data_source_id INTEGER NOT NULL,
            x REAL NOT NULL DEFAULT 0,
            y REAL NOT NULL DEFAULT 0,
            width REAL NOT NULL DEFAULT 300,
            height REAL NOT NULL DEFAULT 200,
            PRIMARY KEY (screen_id, data_source_id)
        );
        CREATE TABLE IF NOT EXISTS twin_connections (
            screen_id TEXT NOT NULL,
            from_id TEXT NOT NULL,
            to_id TEXT NOT NULL,
            PRIMARY KEY (screen_id, from_id, to_id)
        );
        CREATE TABLE IF NOT EXISTS modbus_devices (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            ip TEXT NOT NULL,
            port INTEGER NOT NULL DEFAULT 502,
            unit_id INTEGER NOT NULL DEFAULT 1,
            interval_ms INTEGER NOT NULL DEFAULT 1000,
            endianness TEXT NOT NULL DEFAULT 'big',
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS modbus_registers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id TEXT NOT NULL REFERENCES modbus_devices(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            address INTEGER NOT NULL,
            register_type TEXT NOT NULL DEFAULT 'float32',
            scale REAL NOT NULL DEFAULT 1.0,
            sort_order INTEGER NOT NULL DEFAULT 0,
            UNIQUE(device_id, name)
        );
        CREATE INDEX IF NOT EXISTS idx_modbus_registers_device
            ON modbus_registers(device_id);
        CREATE TABLE IF NOT EXISTS scada_equipment (
            screen_id TEXT NOT NULL,
            equipment_id TEXT NOT NULL,
            equipment_type TEXT NOT NULL,
            equipment_name TEXT NOT NULL DEFAULT '',
            x REAL NOT NULL DEFAULT 0,
            y REAL NOT NULL DEFAULT 0,
            width REAL NOT NULL DEFAULT 100,
            height REAL NOT NULL DEFAULT 60,
            rotation REAL NOT NULL DEFAULT 0,
            sensor_bindings TEXT NOT NULL DEFAULT '[]',
            PRIMARY KEY (screen_id, equipment_id)
        );
        CREATE TABLE IF NOT EXISTS scada_connections (
            screen_id TEXT NOT NULL,
            connection_id TEXT NOT NULL,
            from_equipment TEXT NOT NULL,
            to_equipment TEXT NOT NULL,
            waypoints TEXT NOT NULL DEFAULT '[]',
            connection_type TEXT NOT NULL DEFAULT 'power',
            PRIMARY KEY (screen_id, connection_id)
        );
        CREATE TABLE IF NOT EXISTS telemetry_hourly (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id TEXT NOT NULL,
            metric TEXT NOT NULL,
            bucket_start INTEGER NOT NULL,
            avg_value REAL NOT NULL,
            min_value REAL NOT NULL,
            max_value REAL NOT NULL,
            sample_count INTEGER NOT NULL,
            UNIQUE(device_id, metric, bucket_start)
        );
        CREATE INDEX IF NOT EXISTS idx_hourly_dev_metric
            ON telemetry_hourly(device_id, metric, bucket_start);
        CREATE TABLE IF NOT EXISTS telemetry_daily (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id TEXT NOT NULL,
            metric TEXT NOT NULL,
            bucket_start INTEGER NOT NULL,
            avg_value REAL NOT NULL,
            min_value REAL NOT NULL,
            max_value REAL NOT NULL,
            sample_count INTEGER NOT NULL,
            UNIQUE(device_id, metric, bucket_start)
        );
        CREATE INDEX IF NOT EXISTS idx_daily_dev_metric
            ON telemetry_daily(device_id, metric, bucket_start);
        CREATE TABLE IF NOT EXISTS flows (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            enabled INTEGER DEFAULT 1,
            trigger_config TEXT NOT NULL,
            nodes TEXT NOT NULL,
            created_at INTEGER,
            updated_at INTEGER
        );
        CREATE TABLE IF NOT EXISTS flow_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            flow_id TEXT NOT NULL,
            started_at INTEGER,
            duration_ms INTEGER,
            success INTEGER,
            error TEXT,
            output TEXT
        );
        CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            action TEXT NOT NULL,
            username TEXT NOT NULL,
            resource TEXT NOT NULL,
            details TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_audit_log_created
            ON audit_log(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_audit_log_user
            ON audit_log(username);
        CREATE TABLE IF NOT EXISTS notification_settings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_key TEXT NOT NULL,
            channel_type TEXT NOT NULL CHECK(channel_type IN ('email', 'webhook', 'push')),
            config TEXT NOT NULL DEFAULT '[]',
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_notification_settings_source
            ON notification_settings(source_key);
        CREATE TABLE IF NOT EXISTS api_keys (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            key_hash TEXT NOT NULL UNIQUE,
            key_prefix TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            created_by TEXT NOT NULL,
            last_used_at INTEGER,
            expires_at INTEGER,
            permissions TEXT NOT NULL DEFAULT '[]'
        );
        CREATE INDEX IF NOT EXISTS idx_api_keys_hash
            ON api_keys(key_hash);
        CREATE TABLE IF NOT EXISTS opcua_devices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            security_policy TEXT NOT NULL DEFAULT 'none',
            security_mode TEXT NOT NULL DEFAULT 'none',
            username TEXT,
            password TEXT,
            interval_ms INTEGER NOT NULL DEFAULT 1000,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS opcua_nodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id INTEGER NOT NULL REFERENCES opcua_devices(id) ON DELETE CASCADE,
            node_id TEXT NOT NULL,
            name TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            data_type TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL
        );",
    )
    .context("Failed to create schema")?;

    // Migration: add chart_type column
    let _ = conn.execute_batch("ALTER TABLE data_sources ADD COLUMN chart_type TEXT NOT NULL DEFAULT 'sparkline'");

    // Migration: add equipment_name column to twin_layouts
    let _ = conn.execute_batch("ALTER TABLE twin_layouts ADD COLUMN equipment_name TEXT NOT NULL DEFAULT ''");

    // Migration: rename legacy tables from sensor → data_source
    let _ = conn.execute_batch("ALTER TABLE sensors RENAME TO data_sources");
    let _ = conn.execute_batch("ALTER TABLE screen_sensors RENAME TO screen_data_sources");
    let _ = conn.execute_batch("ALTER TABLE screen_data_sources RENAME COLUMN sensor_id TO data_source_id");
    let _ = conn.execute_batch("ALTER TABLE user_shortcuts RENAME COLUMN sensor_id TO data_source_id");

    // Migration: add role column to users
    let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'user'");
    let _ = conn.execute_batch("UPDATE users SET role = 'admin' WHERE username = 'admin'");

    // Migration: add cooldown_until column to alarms
    let _ = conn.execute_batch("ALTER TABLE alarms ADD COLUMN cooldown_until INTEGER");

    // Migration: add register_area column to modbus_registers
    let _ = conn.execute_batch("ALTER TABLE modbus_registers ADD COLUMN register_area TEXT NOT NULL DEFAULT 'holding'");

    // Migration: add must_change_password column to users
    let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN must_change_password INTEGER NOT NULL DEFAULT 0");
    // Force existing admin users to change password (they likely have the default "admin" password)
    let _ = conn.execute_batch("UPDATE users SET must_change_password = 1 WHERE username = 'admin'");

        // Migration: add account lockout columns to users
        let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN failed_attempts INTEGER NOT NULL DEFAULT 0");
        let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN locked_until INTEGER");

        // Migration: add refresh_tokens table
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS refresh_tokens (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token_hash TEXT NOT NULL UNIQUE,
                username TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                revoked_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_refresh_tokens_username ON refresh_tokens(username);
            CREATE INDEX IF NOT EXISTS idx_refresh_tokens_hash ON refresh_tokens(token_hash);"
        );

    // Migration: add license_activation table
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS license_activation (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            license_key TEXT NOT NULL UNIQUE,
            fingerprint_hash TEXT NOT NULL,
            fingerprint_hostname TEXT NOT NULL,
            fingerprint_cpu TEXT NOT NULL,
            fingerprint_disk_serial TEXT NOT NULL,
            fingerprint_mac TEXT NOT NULL,
            fingerprint_machine_id TEXT NOT NULL,
            activation_blob BLOB NOT NULL,
            activated_at TEXT NOT NULL,
            expires_at TEXT,
            tier TEXT,
            device_limit INTEGER
        );",
    );

    // Migration: add tier and device_limit columns to existing databases
    let _ = conn.execute_batch(
        "ALTER TABLE license_activation ADD COLUMN tier TEXT;",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE license_activation ADD COLUMN device_limit INTEGER;",
    );

        Ok(())
}

// ── Modbus device management ──

impl Storage {
    pub async fn is_yaml_migrated(&self) -> Result<bool> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'modbus_yaml_migrated'")?;
            let result = stmt.query_row([], |row| row.get::<_, String>(0)).ok();
            Ok(result.as_deref() == Some("true"))
        }).await.context("Task join failed")?
    }

    pub async fn mark_yaml_migrated(&self) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('modbus_yaml_migrated', 'true')",
                [],
            ).context("Mark yaml migrated failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    pub async fn import_yaml_devices(&self, devices: &[DeviceConfig]) -> Result<()> {
        let conn = self.conn.clone();
        let devices = devices.to_vec();
        let now = crate::timeutil::now_secs();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;

            for device in &devices {
                let endianness = match device.endianness {
                    crate::models::Endianness::Big => "big",
                    crate::models::Endianness::Little => "little",
                    crate::models::Endianness::Swapped => "swapped",
                };
                tx.execute(
                    "INSERT OR IGNORE INTO modbus_devices (id, name, ip, port, unit_id, interval_ms, endianness, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8)",
                    rusqlite::params![device.id, device.id, device.ip, device.port, device.unit_id, device.interval_ms, endianness, now],
                ).context("Insert modbus device failed")?;

                for (i, reg) in device.registers.iter().enumerate() {
                    let reg_type = match reg.reg_type {
                        crate::models::RegisterType::Int16 => "int16",
                        crate::models::RegisterType::Uint16 => "uint16",
                        crate::models::RegisterType::Int32 => "int32",
                        crate::models::RegisterType::Float32 => "float32",
                    };
                    tx.execute(
                        "INSERT OR IGNORE INTO modbus_registers (device_id, name, address, register_type, register_area, scale, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        rusqlite::params![device.id, reg.name, reg.address, reg_type, reg.register_area, reg.scale, i as i64],
                    ).context("Insert modbus register failed")?;
                }
            }

            tx.commit().context("Commit failed")
        }).await.context("Task join failed")?
    }

    pub async fn list_modbus_devices(&self) -> Result<Vec<ModbusDevice>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, name, ip, port, unit_id, interval_ms, endianness, enabled, created_at, updated_at FROM modbus_devices ORDER BY created_at"
            ).context("Prepare failed")?;
            let rows = stmt.query_map([], |row| {
                Ok(ModbusDevice {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    ip: row.get(2)?,
                    port: row.get(3)?,
                    unit_id: row.get(4)?,
                    interval_ms: row.get(5)?,
                    endianness: row.get(6)?,
                    enabled: row.get::<_, i64>(7)? != 0,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            }).context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>().context("Row mapping failed")
        }).await.context("Task join failed")?
    }

    pub async fn get_modbus_device(&self, id: &str) -> Result<Option<ModbusDevice>> {
        let conn = self.conn.clone();
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, name, ip, port, unit_id, interval_ms, endianness, enabled, created_at, updated_at FROM modbus_devices WHERE id = ?1"
            ).context("Prepare failed")?;
            let result = stmt.query_row(rusqlite::params![id], |row| {
                Ok(ModbusDevice {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    ip: row.get(2)?,
                    port: row.get(3)?,
                    unit_id: row.get(4)?,
                    interval_ms: row.get(5)?,
                    endianness: row.get(6)?,
                    enabled: row.get::<_, i64>(7)? != 0,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            }).ok();
            Ok(result)
        }).await.context("Task join failed")?
    }

    pub async fn create_modbus_device(
        &self,
        id: &str,
        name: &str,
        ip: &str,
        port: u16,
        unit_id: u8,
        interval_ms: u64,
        endianness: &str,
        registers: &[ModbusRegisterInput],
    ) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let name = name.to_string();
        let ip = ip.to_string();
        let endianness = endianness.to_string();
        let registers: Vec<(String, u16, String, String, f64)> = registers.iter().map(|r| {
            (r.name.clone(), r.address, r.register_type.clone(), r.register_area.clone(), r.scale)
        }).collect();
        let now = crate::timeutil::now_secs();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;

            tx.execute(
                "INSERT INTO modbus_devices (id, name, ip, port, unit_id, interval_ms, endianness, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8)",
                rusqlite::params![id, name, ip, port, unit_id, interval_ms, endianness, now],
            ).context("Insert modbus device failed")?;

            for (i, (rname, addr, rtype, rarea, scale)) in registers.iter().enumerate() {
                tx.execute(
                    "INSERT INTO modbus_registers (device_id, name, address, register_type, register_area, scale, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![id, rname, addr, rtype, rarea, scale, i as i64],
                ).context("Insert modbus register failed")?;
            }

            tx.commit().context("Commit failed")
        }).await.context("Task join failed")?
    }

    pub async fn update_modbus_device(&self, id: &str, fields: &serde_json::Value) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let now = crate::timeutil::now_secs();
        let fields = fields.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let obj = fields.as_object().context("Expected JSON object")?;

            // Extract all optional values
            let name = obj.get("name").and_then(|v| v.as_str());
            let ip = obj.get("ip").and_then(|v| v.as_str());
            let endianness = obj.get("endianness").and_then(|v| v.as_str());
            let port: Option<u16> = obj.get("port").and_then(|v| v.as_u64()).map(|v| v as u16);
            let unit_id: Option<u8> = obj.get("unit_id").and_then(|v| v.as_u64()).map(|v| v as u8);
            let interval_ms: Option<u64> = obj.get("interval_ms").and_then(|v| v.as_u64());
            let enabled: Option<bool> = obj.get("enabled").and_then(|v| v.as_bool());

            // Build SET clauses and params list
            let mut sets = Vec::new();
            let mut p: Vec<rusqlite::types::Value> = Vec::new();

            if let Some(v) = name { sets.push("name = ?"); p.push(rusqlite::types::Value::from(v.to_string())); }
            if let Some(v) = ip { sets.push("ip = ?"); p.push(rusqlite::types::Value::from(v.to_string())); }
            if let Some(v) = endianness { sets.push("endianness = ?"); p.push(rusqlite::types::Value::from(v.to_string())); }
            if let Some(v) = port { sets.push("port = ?"); p.push(rusqlite::types::Value::from(v as i64)); }
            if let Some(v) = unit_id { sets.push("unit_id = ?"); p.push(rusqlite::types::Value::from(v as i64)); }
            if let Some(v) = interval_ms { sets.push("interval_ms = ?"); p.push(rusqlite::types::Value::from(v as i64)); }
            if let Some(v) = enabled { sets.push("enabled = ?"); p.push(rusqlite::types::Value::from(v as i64)); }

            if sets.is_empty() { return Ok(()); }
            sets.push("updated_at = ?");
            p.push(rusqlite::types::Value::from(now));
            p.push(rusqlite::types::Value::from(id));

            let sql = format!("UPDATE modbus_devices SET {} WHERE id = ?", sets.join(", "));
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = p.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();
            conn.execute(&sql, param_refs.as_slice()).context("Update modbus device failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    pub async fn delete_modbus_device(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM modbus_devices WHERE id = ?1", rusqlite::params![id])
                .context("Delete modbus device failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    pub async fn list_modbus_registers(&self, device_id: &str) -> Result<Vec<ModbusRegister>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, device_id, name, address, register_type, register_area, scale, sort_order FROM modbus_registers WHERE device_id = ?1 ORDER BY sort_order"
            ).context("Prepare failed")?;
            let rows = stmt.query_map(rusqlite::params![device_id], |row| {
                Ok(ModbusRegister {
                    id: row.get(0)?,
                    device_id: row.get(1)?,
                    name: row.get(2)?,
                    address: row.get(3)?,
                    register_type: row.get(4)?,
                    register_area: row.get(5)?,
                    scale: row.get(6)?,
                    sort_order: row.get(7)?,
                })
            }).context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>().context("Row mapping failed")
        }).await.context("Task join failed")?
    }

    pub async fn replace_modbus_registers(&self, device_id: &str, registers: &[ModbusRegisterInput]) -> Result<()> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        let registers: Vec<(String, u16, String, String, f64)> = registers.iter().map(|r| {
            (r.name.clone(), r.address, r.register_type.clone(), r.register_area.clone(), r.scale)
        }).collect();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            tx.execute("DELETE FROM modbus_registers WHERE device_id = ?1", rusqlite::params![device_id])
                .context("Delete registers failed")?;
            for (i, (rname, addr, rtype, rarea, scale)) in registers.iter().enumerate() {
                tx.execute(
                    "INSERT INTO modbus_registers (device_id, name, address, register_type, register_area, scale, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![device_id, rname, addr, rtype, rarea, scale, i as i64],
                ).context("Insert register failed")?;
            }
            tx.commit().context("Commit failed")
        }).await.context("Task join failed")?
    }

    pub async fn get_all_device_configs(&self) -> Result<Vec<DeviceConfig>> {
        let devices = self.list_modbus_devices().await?;
        let mut configs = Vec::new();
        for device in &devices {
            if !device.enabled { continue; }
            let registers = self.list_modbus_registers(&device.id).await?;
            configs.push(device.to_device_config(&registers));
        }
        Ok(configs)
    }

    // ── OPC-UA device management ──

    pub async fn create_opcua_device(
        &self,
        name: &str,
        url: &str,
        security_policy: &OpcUaSecurityPolicy,
        security_mode: &OpcUaSecurityMode,
        username: Option<&str>,
        password: Option<&str>,
        interval_ms: u64,
        enabled: bool,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let url = url.to_string();
        let security_policy = serde_json::to_string(security_policy).unwrap_or_else(|_| "none".to_string());
        let security_mode = serde_json::to_string(security_mode).unwrap_or_else(|_| "none".to_string());
        let username = username.map(|s| s.to_string());
        let password = password.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO opcua_devices (name, url, security_policy, security_mode, username, password, interval_ms, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![name, url, security_policy, security_mode, username, password, interval_ms, enabled as i64, now],
            )
            .context("Insert opcua device failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_opcua_devices(&self) -> Result<Vec<OpcUaDevice>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, url, security_policy, security_mode, username, password, interval_ms, enabled, created_at FROM opcua_devices ORDER BY created_at"
                )
                .context("Prepare list_opcua_devices failed")?;

            let rows = stmt.query_map([], |row| {
                let policy_str: String = row.get(3)?;
                let mode_str: String = row.get(4)?;
                Ok(OpcUaDevice {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    security_policy: serde_json::from_str(&policy_str).unwrap_or(OpcUaSecurityPolicy::None),
                    security_mode: serde_json::from_str(&mode_str).unwrap_or(OpcUaSecurityMode::None),
                    username: row.get(5)?,
                    password: row.get(6)?,
                    interval_ms: row.get(7)?,
                    enabled: row.get::<_, i64>(8)? != 0,
                    created_at: row.get(9)?,
                })
            })
            .context("Query list_opcua_devices failed")?;

            let mut devices = Vec::new();
            for row in rows {
                devices.push(row.context("Row list_opcua_devices failed")?);
            }
            Ok(devices)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_opcua_device(
        &self,
        id: i64,
        name: Option<&str>,
        url: Option<&str>,
        security_policy: Option<&OpcUaSecurityPolicy>,
        security_mode: Option<&OpcUaSecurityMode>,
        username: Option<Option<&str>>,
        password: Option<Option<&str>>,
        interval_ms: Option<u64>,
        enabled: Option<bool>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let name = name.map(|s| s.to_string());
        let url = url.map(|s| s.to_string());
        let security_policy = security_policy.map(|p| serde_json::to_string(p).unwrap_or_else(|_| "none".to_string()));
        let security_mode = security_mode.map(|m| serde_json::to_string(m).unwrap_or_else(|_| "none".to_string()));
        let username = username.map(|o| o.map(|s| s.to_string()));
        let password = password.map(|o| o.map(|s| s.to_string()));

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            if let Some(name) = &name {
                conn.execute("UPDATE opcua_devices SET name = ?1 WHERE id = ?2", rusqlite::params![name, id])
                    .context("Update opcua device name failed")?;
            }
            if let Some(url) = &url {
                conn.execute("UPDATE opcua_devices SET url = ?1 WHERE id = ?2", rusqlite::params![url, id])
                    .context("Update opcua device url failed")?;
            }
            if let Some(policy) = &security_policy {
                conn.execute("UPDATE opcua_devices SET security_policy = ?1 WHERE id = ?2", rusqlite::params![policy, id])
                    .context("Update opcua device security_policy failed")?;
            }
            if let Some(mode) = &security_mode {
                conn.execute("UPDATE opcua_devices SET security_mode = ?1 WHERE id = ?2", rusqlite::params![mode, id])
                    .context("Update opcua device security_mode failed")?;
            }
            if let Some(ref username) = username {
                conn.execute("UPDATE opcua_devices SET username = ?1 WHERE id = ?2", rusqlite::params![username, id])
                    .context("Update opcua device username failed")?;
            }
            if let Some(ref password) = password {
                conn.execute("UPDATE opcua_devices SET password = ?1 WHERE id = ?2", rusqlite::params![password, id])
                    .context("Update opcua device password failed")?;
            }
            if let Some(interval_ms) = interval_ms {
                conn.execute("UPDATE opcua_devices SET interval_ms = ?1 WHERE id = ?2", rusqlite::params![interval_ms, id])
                    .context("Update opcua device interval_ms failed")?;
            }
            if let Some(enabled) = enabled {
                conn.execute("UPDATE opcua_devices SET enabled = ?1 WHERE id = ?2", rusqlite::params![enabled as i64, id])
                    .context("Update opcua device enabled failed")?;
            }
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_opcua_device(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM opcua_devices WHERE id = ?1", rusqlite::params![id])
                .context("Delete opcua device failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }

    pub async fn create_opcua_node(
        &self,
        device_id: i64,
        node_id: &str,
        name: &str,
        display_name: &str,
        data_type: &str,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let node_id = node_id.to_string();
        let name = name.to_string();
        let display_name = display_name.to_string();
        let data_type = data_type.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO opcua_nodes (device_id, node_id, name, display_name, data_type, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![device_id, node_id, name, display_name, data_type, now],
            )
            .context("Insert opcua node failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_opcua_nodes(&self, device_id: i64) -> Result<Vec<OpcUaNode>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, device_id, node_id, name, display_name, data_type, created_at FROM opcua_nodes WHERE device_id = ?1 ORDER BY created_at"
                )
                .context("Prepare list_opcua_nodes failed")?;

            let rows = stmt.query_map(rusqlite::params![device_id], |row| {
                Ok(OpcUaNode {
                    id: row.get(0)?,
                    device_id: row.get(1)?,
                    node_id: row.get(2)?,
                    name: row.get(3)?,
                    display_name: row.get(4)?,
                    data_type: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .context("Query list_opcua_nodes failed")?;

            let mut nodes = Vec::new();
            for row in rows {
                nodes.push(row.context("Row list_opcua_nodes failed")?);
            }
            Ok(nodes)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_opcua_node(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM opcua_nodes WHERE id = ?1", rusqlite::params![id])
                .context("Delete opcua node failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }

    /// Load all enabled OPC-UA devices with their nodes and convert to runtime configs.
    pub async fn get_all_opcua_device_configs(&self) -> Result<Vec<OpcUaDeviceConfig>> {
        let devices = self.list_opcua_devices().await?;
        let mut configs = Vec::new();
        for device in &devices {
            if !device.enabled { continue; }
            let nodes = self.list_opcua_nodes(device.id).await?;
            let device_key = format!("opcua_{}", device.id);
            configs.push(OpcUaDeviceConfig {
                id: device_key,
                url: device.url.clone(),
                security_policy: device.security_policy.clone(),
                security_mode: device.security_mode.clone(),
                username: device.username.clone(),
                password: device.password.clone(),
                interval_ms: device.interval_ms,
                nodes: nodes.iter().map(|n| OpcUaNodeConfig {
                    node_id: n.node_id.clone(),
                    name: n.name.clone(),
                    display_name: n.display_name.clone(),
                    data_type: n.data_type.clone(),
                }).collect(),
            });
        }
        Ok(configs)
    }
}

// ── API Key management ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyInfo {
    pub id: i64,
    pub name: String,
    pub key_prefix: String,
    pub created_at: i64,
    pub created_by: String,
    pub last_used_at: Option<i64>,
    pub expires_at: Option<i64>,
    pub permissions: String,
}

impl Storage {
    /// Create a new API key. Returns (id, plain_text_key). The plain key is shown only once.
    pub async fn create_api_key(
        &self,
        name: &str,
        created_by: &str,
        expires_at: Option<i64>,
        permissions: &str,
    ) -> Result<(i64, String)> {
        use rand::Rng;
        use sha2::{Sha256, Digest};

        // Generate 32 random bytes, hex-encoded = 64-char key
        let key_bytes: [u8; 32] = rand::rng().random();
        let plain_key = hex::encode(key_bytes);

        // Hash with SHA-256 for storage
        let mut hasher = Sha256::new();
        hasher.update(plain_key.as_bytes());
        let key_hash = hex::encode(hasher.finalize());

        // Prefix is first 8 chars of the plain key
        let key_prefix = plain_key[..8].to_string();

        let now = crate::timeutil::now_secs();
        let name = name.to_string();
        let created_by = created_by.to_string();
        let permissions = permissions.to_string();

        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO api_keys (name, key_hash, key_prefix, created_at, created_by, expires_at, permissions)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![name, key_hash, key_prefix, now, created_by, expires_at, permissions],
            ).context("Insert api_key failed")?;
            let id = conn.last_insert_rowid();
            Ok((id, plain_key))
        }).await.context("Task join failed")?
    }

    /// Validate an API key by hashing it and looking it up. Returns (id, permissions) if valid.
    pub async fn validate_api_key(&self, key: &str) -> Result<Option<(i64, String)>> {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let key_hash = hex::encode(hasher.finalize());

        let now = crate::timeutil::now_secs();
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, permissions, expires_at FROM api_keys WHERE key_hash = ?1"
            ).context("Prepare failed")?;
            let result = stmt.query_row(rusqlite::params![key_hash], |row| {
                let id: i64 = row.get(0)?;
                let permissions: String = row.get(1)?;
                let expires_at: Option<i64> = row.get(2)?;
                Ok((id, permissions, expires_at))
            }).ok();

            match result {
                Some((id, permissions, expires_at)) => {
                    // Check expiry
                    if let Some(exp) = expires_at {
                        if exp < now {
                            return Ok(None);
                        }
                    }
                    // Update last_used_at
                    let _ = conn.execute(
                        "UPDATE api_keys SET last_used_at = ?1 WHERE id = ?2",
                        rusqlite::params![now, id],
                    );
                    Ok(Some((id, permissions)))
                }
                None => Ok(None),
            }
        }).await.context("Task join failed")?
    }

    /// List all API keys (never includes the full key hash).
    pub async fn list_api_keys(&self) -> Result<Vec<ApiKeyInfo>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, name, key_prefix, created_at, created_by, last_used_at, expires_at, permissions
                 FROM api_keys ORDER BY created_at DESC"
            ).context("Prepare failed")?;
            let rows = stmt.query_map([], |row| {
                Ok(ApiKeyInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    key_prefix: row.get(2)?,
                    created_at: row.get(3)?,
                    created_by: row.get(4)?,
                    last_used_at: row.get(5)?,
                    expires_at: row.get(6)?,
                    permissions: row.get(7)?,
                })
            }).context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>().context("Row mapping failed")
        }).await.context("Task join failed")?
    }

    /// Revoke (delete) an API key by id.
    pub async fn revoke_api_key(&self, id: i64) -> Result<bool> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let affected = conn.execute(
                "DELETE FROM api_keys WHERE id = ?1",
                rusqlite::params![id],
            ).context("Delete api_key failed")?;
            Ok(affected > 0)
        }).await.context("Task join failed")?
    }
}

// ── License Activation ──

impl Storage {
    pub async fn get_license_activation(&self) -> Result<Option<crate::models::LicenseActivationRow>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT license_key, fingerprint_hash, activation_blob, fingerprint_hostname, fingerprint_cpu, fingerprint_disk_serial, fingerprint_mac, fingerprint_machine_id, activated_at, expires_at, tier, device_limit FROM license_activation ORDER BY id DESC LIMIT 1")
                .context("Prepare failed")?;
            let result = stmt.query_row([], |row| {
                Ok(crate::models::LicenseActivationRow {
                    license_key: row.get(0)?,
                    fingerprint_hash: row.get(1)?,
                    activation_blob: row.get(2)?,
                    fingerprint_hostname: row.get(3)?,
                    fingerprint_cpu: row.get(4)?,
                    fingerprint_disk_serial: row.get(5)?,
                    fingerprint_mac: row.get(6)?,
                    fingerprint_machine_id: row.get(7)?,
                    activated_at: row.get(8)?,
                    expires_at: row.get(9)?,
                    tier: row.get(10)?,
                    device_limit: row.get(11)?,
                })
            }).ok();
            Ok(result)
        }).await.context("Task join failed")?
    }

    pub async fn store_license_activation(&self, row: &crate::models::LicenseActivationRow) -> Result<()> {
        let conn = self.conn.clone();
        let row = row.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            // Clear any existing activation first — only one license can be active at a time
            let _ = conn.execute("DELETE FROM license_activation", []);
            conn.execute(
                "INSERT INTO license_activation (license_key, fingerprint_hash, fingerprint_hostname, fingerprint_cpu, fingerprint_disk_serial, fingerprint_mac, fingerprint_machine_id, activation_blob, activated_at, expires_at, tier, device_limit)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                rusqlite::params![
                    row.license_key,
                    row.fingerprint_hash,
                    row.fingerprint_hostname,
                    row.fingerprint_cpu,
                    row.fingerprint_disk_serial,
                    row.fingerprint_mac,
                    row.fingerprint_machine_id,
                    row.activation_blob,
                    row.activated_at,
                    row.expires_at,
                    row.tier,
                    row.device_limit,
                ],
            ).context("Store license activation failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    pub async fn clear_license_activation(&self) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM license_activation", [])
                .context("Clear license activation failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_and_query() {
        let storage = Storage::new_in_memory().unwrap();

        let records = vec![
            ("dev1".into(), 1000i64, "voltage".into(), 220.5f64),
            ("dev1".into(), 1060i64, "voltage".into(), 221.0f64),
            ("dev1".into(), 1120i64, "voltage".into(), 219.8f64),
        ];
        storage.insert_batch(&records).await.unwrap();

        let points = storage.query_history("dev1", "voltage", 0, 2000).await.unwrap();

        assert_eq!(points.len(), 3);
        assert_eq!(points[0], DataPoint { timestamp: 1000, value: 220.5 });
        assert_eq!(points[1], DataPoint { timestamp: 1060, value: 221.0 });
        assert_eq!(points[2], DataPoint { timestamp: 1120, value: 219.8 });
    }

    #[tokio::test]
    async fn test_query_filters_by_device_and_metric() {
        let storage = Storage::new_in_memory().unwrap();

        let records = vec![
            ("dev1".into(), 1000, "voltage".into(), 220.0),
            ("dev1".into(), 1000, "current".into(), 5.0),
            ("dev2".into(), 1000, "voltage".into(), 380.0),
        ];
        storage.insert_batch(&records).await.unwrap();

        let points = storage.query_history("dev1", "voltage", 0, 2000).await.unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].value, 220.0);
    }

    #[tokio::test]
    async fn test_query_time_range() {
        let storage = Storage::new_in_memory().unwrap();

        let records = vec![
            ("dev1".into(), 1000, "voltage".into(), 220.0),
            ("dev1".into(), 2000, "voltage".into(), 221.0),
            ("dev1".into(), 3000, "voltage".into(), 222.0),
        ];
        storage.insert_batch(&records).await.unwrap();

        let points = storage.query_history("dev1", "voltage", 1500, 2500).await.unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].value, 221.0);
    }

    #[tokio::test]
    async fn test_empty_batch() {
        let storage = Storage::new_in_memory().unwrap();
        let records: Vec<(String, i64, String, f64)> = vec![];
        storage.insert_batch(&records).await.unwrap();
    }

    #[tokio::test]
    async fn test_no_results() {
        let storage = Storage::new_in_memory().unwrap();
        let points = storage.query_history("ghost", "voltage", 0, 9999).await.unwrap();
        assert!(points.is_empty());
    }

    #[tokio::test]
    async fn test_downsample_raw_to_hourly() {
        let storage = Storage::new_in_memory().unwrap();

        // Insert 2 hours of data, 2 points per hour
        let base_ts: i64 = 3600; // start at 1 hour
        let mut records = Vec::new();
        for hour in 0..2 {
            let ts = base_ts + hour * 3600;
            records.push(("dev1".into(), ts, "voltage".into(), 220.0 + hour as f64));
            records.push(("dev1".into(), ts + 60, "voltage".into(), 221.0 + hour as f64));
        }
        storage.insert_batch(&records).await.unwrap();

        // Downsample everything (cutoff far in future)
        let count = storage.downsample_raw_to_hourly(base_ts + 7200).await.unwrap();
        assert_eq!(count, 2); // 2 hourly buckets

        // Verify aggregated values via direct query
        let conn = storage.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT bucket_start, avg_value, min_value, max_value, sample_count
             FROM telemetry_hourly WHERE device_id = 'dev1' ORDER BY bucket_start"
        ).unwrap();
        let rows: Vec<(i64, f64, f64, f64, i64)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(rows.len(), 2);
        // First hour: avg of 220.0 and 221.0 = 220.5
        assert!((rows[0].1 - 220.5).abs() < 0.01);
        assert!((rows[0].2 - 220.0).abs() < 0.01);
        assert!((rows[0].3 - 221.0).abs() < 0.01);
        assert_eq!(rows[0].4, 2);
    }

    #[tokio::test]
    async fn test_downsample_hourly_to_daily() {
        let storage = Storage::new_in_memory().unwrap();

        // Manually insert hourly data on the same day (86400 = day 1)
        let conn = storage.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO telemetry_hourly (device_id, metric, bucket_start, avg_value, min_value, max_value, sample_count)
             VALUES ('dev1', 'voltage', 86400, 220.0, 219.0, 221.0, 12)", [],
        ).unwrap();
        conn.execute(
            "INSERT INTO telemetry_hourly (device_id, metric, bucket_start, avg_value, min_value, max_value, sample_count)
             VALUES ('dev1', 'voltage', 90000, 222.0, 221.0, 223.0, 12)", [],
        ).unwrap();
        drop(conn);

        // Both hourly buckets fall on the same day (86400), cutoff well past them
        let count = storage.downsample_hourly_to_daily(100000).await.unwrap();
        assert_eq!(count, 1); // 1 daily bucket

        let conn = storage.conn.lock().unwrap();
        let (avg, min, max, cnt): (f64, f64, f64, i64) = conn.query_row(
            "SELECT avg_value, min_value, max_value, sample_count FROM telemetry_daily WHERE device_id = 'dev1'",
            [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).unwrap();
        // avg of 220.0 and 222.0 = 221.0
        assert!((avg - 221.0).abs() < 0.01);
        assert!((min - 219.0).abs() < 0.01);
        assert!((max - 223.0).abs() < 0.01);
        assert_eq!(cnt, 24);
    }

    #[tokio::test]
    async fn test_delete_raw_older_than() {
        let storage = Storage::new_in_memory().unwrap();
        let records = vec![
            ("dev1".into(), 1000i64, "voltage".into(), 220.0f64),
            ("dev1".into(), 2000i64, "voltage".into(), 221.0f64),
            ("dev1".into(), 3000i64, "voltage".into(), 222.0f64),
        ];
        storage.insert_batch(&records).await.unwrap();

        let deleted = storage.delete_raw_older_than(2000).await.unwrap();
        assert_eq!(deleted, 1);

        let points = storage.query_history("dev1", "voltage", 0, 9999).await.unwrap();
        assert_eq!(points.len(), 2);
    }

    #[tokio::test]
    async fn test_api_key_create_and_validate() {
        let storage = Storage::new_in_memory().unwrap();

        let (id, plain_key) = storage.create_api_key("test-key", "admin", None, "[]").await.unwrap();
        assert!(id > 0);
        assert_eq!(plain_key.len(), 64); // 32 bytes hex-encoded

        // Validate the key
        let result = storage.validate_api_key(&plain_key).await.unwrap();
        assert!(result.is_some());
        let (key_id, permissions) = result.unwrap();
        assert_eq!(key_id, id);
        assert_eq!(permissions, "[]");

        // Wrong key should fail
        let result = storage.validate_api_key("wrong_key").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_api_key_list_and_revoke() {
        let storage = Storage::new_in_memory().unwrap();

        let (id1, _) = storage.create_api_key("key-one", "admin", None, "[]").await.unwrap();
        let (id2, _) = storage.create_api_key("key-two", "admin", None, "[\"data:read\"]").await.unwrap();

        let keys = storage.list_api_keys().await.unwrap();
        assert_eq!(keys.len(), 2);

        // Verify key prefix is first 8 chars and key_hash is never returned
        for k in &keys {
            assert_eq!(k.key_prefix.len(), 8);
        }

        // Revoke one
        let deleted = storage.revoke_api_key(id1).await.unwrap();
        assert!(deleted);

        let keys = storage.list_api_keys().await.unwrap();
        assert_eq!(keys.len(), 1);

        // Revoking again should return false
        let deleted = storage.revoke_api_key(id1).await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_api_key_expiry() {
        let storage = Storage::new_in_memory().unwrap();

        // Create key that expired in the past
        let (_, plain_key) = storage
            .create_api_key("expired-key", "admin", Some(1000), "[]")
            .await
            .unwrap();

        // Should not validate (expired)
        let result = storage.validate_api_key(&plain_key).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_api_key_last_used_at_updated() {
        let storage = Storage::new_in_memory().unwrap();

        let (id, plain_key) = storage.create_api_key("usage-key", "admin", None, "[]").await.unwrap();

        // Initially last_used_at should be None
        let keys = storage.list_api_keys().await.unwrap();
        assert!(keys[0].last_used_at.is_none());

        // Validate (which updates last_used_at)
        storage.validate_api_key(&plain_key).await.unwrap();

        let keys = storage.list_api_keys().await.unwrap();
        assert!(keys[0].last_used_at.is_some());
    }
}
