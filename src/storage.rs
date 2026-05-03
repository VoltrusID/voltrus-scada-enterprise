use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::models::{
    DataPoint, Device, ScreenLayoutRow, ScreenDataSource, DataSource, Setting, TwinLayout, TwinConnection, User, UserShortcut,
    NavShortcut, Alarm, CanvasLayoutItem,
    ModbusDevice, ModbusRegister, ModbusRegisterInput, DeviceConfig,
    ScadaEquipment, ScadaConnection,
    Flow, FlowRun,
    OpcUaDevice, OpcUaNode, OpcUaDeviceConfig, OpcUaNodeConfig,
    OpcUaSecurityPolicy, OpcUaSecurityMode,
    HubSite,
    Report,
    DowntimeEvent, OeeMetrics, ShiftConfig,
    Role,
    S7Device, S7TagRow,
    EnipDevice, EnipTag, EnipDeviceConfig,
    Camera, CameraSnapshot,
    Annotation, CreateAnnotationRequest, UpdateAnnotationRequest,
    ShiftHandover, CreateShiftHandoverRequest,
    UserNotification, CreateNotificationRequest,
    AlarmAuditEntry, AlarmSummary, AlarmAnalyticsEntry,
    Command, CommandAuditEntry, WritePolicy, WritePolicyUpdate, CommandQuery,
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

    pub async fn get_data_source_by_id(&self, id: i64) -> Result<Option<DataSource>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let result = conn.query_row(
                "SELECT id, key, name, unit, color, min_val, max_val,
                        warn_val, crit_val, base_val, sort_order, decimals,
                        source_type, source_config, device_id, equipment_id, chart_type
                 FROM data_sources WHERE id = ?1",
                rusqlite::params![id],
                |row| {
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
                },
            );
            match result {
                Ok(ds) => Ok(Some(ds)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e).context("Get data source by id failed"),
            }
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

    pub async fn get_screen_data_sources(&self, screen_id: &str) -> Result<Vec<ScreenDataSource>> {
        let conn = self.conn.clone();
        let screen_id = screen_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT screen_id, data_source_id, sort_order FROM screen_data_sources WHERE screen_id = ?1 ORDER BY sort_order")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map(rusqlite::params![screen_id], |row| {
                    Ok(ScreenDataSource {
                        screen_id: row.get(0)?,
                        data_source_id: row.get(1)?,
                        sort_order: row.get(2)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
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

    // ── JIT User Provisioning (SSO) ──

    /// JIT (Just-in-Time) provisions a user with a default "viewer" role.
    /// Creates the user if they don't already exist. Returns the user ID.
    pub async fn jit_provision_user(
        &self,
        username: &str,
        display_name: &str,
        email: &str,
        role: &str,
    ) -> Result<i64> {
        // Check if user already exists
        if let Some(existing) = self.get_user_by_username(username).await? {
            return Ok(existing.id);
        }

        // Create user with a random unusable password (SSO-authenticated)
        let random_password = format!("sso:{}", hex::encode(rand::random::<[u8; 32]>()));
        let password_hash = crate::api::auth::hash_password(&random_password);
        let id = self.create_user(username, &password_hash, "", role).await?;

        tracing::info!(
            "JIT provisioned user: username={}, role={}, email={}",
            username,
            role,
            email
        );

        // Store external auth metadata in settings KV
        let user_meta = serde_json::json!({
            "display_name": display_name,
            "email": email,
            "provisioned_via": "jit",
            "provisioned_at": crate::timeutil::now_secs(),
        });
        self.upsert_setting(
            &format!("user_meta:{}", username),
            &user_meta.to_string(),
        )
        .await?;

        Ok(id)
    }

    /// Gets or creates a user by their external identity (provider + external_id).
    /// If the user doesn't exist, JIT provisions them.
    /// Stores the provider/external_id mapping in the settings KV table.
    pub async fn get_or_create_user_by_external_id(
        &self,
        provider: &str,
        external_id: &str,
        default_role: &str,
    ) -> Result<User> {
        let mapping_key = format!("ext_auth:{}:{}", provider, external_id);

        // Check if we have an existing mapping
        let settings = self.get_settings(&[&mapping_key]).await?;
        if let Some(setting) = settings.into_iter().next() {
            // Found existing mapping — look up the user
            if let Some(user) = self.get_user_by_username(&setting.value).await? {
                return Ok(user);
            }
        }

        // No existing mapping — create a new user
        // Derive a username from the external_id
        let username = format!("{}:{}", provider, external_id);
        let user_id = self
            .jit_provision_user(&username, external_id, external_id, default_role)
            .await?;

        // Store the mapping
        self.upsert_setting(&mapping_key, &username).await?;

        // Retrieve and return the newly created user
        self.get_user_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve JIT-provisioned user"))
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

    // ── Role management ──

    pub async fn create_role(&self, name: &str, permissions: &str) -> Result<i64> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let permissions = permissions.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO roles (name, permissions, created_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![name, permissions, now],
            )
            .context("Insert role failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_roles(&self) -> Result<Vec<Role>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, name, permissions, created_at FROM roles ORDER BY id")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(Role {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        permissions: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_role_by_id(&self, id: i64) -> Result<Option<Role>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let result = conn
                .query_row(
                    "SELECT id, name, permissions, created_at FROM roles WHERE id = ?1",
                    rusqlite::params![id],
                    |row| {
                        Ok(Role {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            permissions: row.get(2)?,
                            created_at: row.get(3)?,
                        })
                    },
                )
                .ok();
            Ok(result)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_role(&self, id: i64, name: Option<&str>, permissions: Option<&str>) -> Result<()> {
        let conn = self.conn.clone();
        let name = name.map(|s| s.to_string());
        let permissions = permissions.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            if let Some(name) = name {
                conn.execute(
                    "UPDATE roles SET name = ?1 WHERE id = ?2",
                    rusqlite::params![name, id],
                )
                .context("Update role name failed")?;
            }
            if let Some(permissions) = permissions {
                conn.execute(
                    "UPDATE roles SET permissions = ?1 WHERE id = ?2",
                    rusqlite::params![permissions, id],
                )
                .context("Update role permissions failed")?;
            }
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_role(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM roles WHERE id = ?1", rusqlite::params![id])
                .context("Delete role failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn assign_user_role(&self, user_id: i64, role_id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE users SET role_id = ?1 WHERE id = ?2",
                rusqlite::params![role_id, user_id],
            )
            .context("Assign user role failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_user_role_permissions(&self, user_id: i64) -> Result<Vec<String>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let permissions_json: Option<String> = conn
                .query_row(
                    "SELECT r.permissions FROM users u LEFT JOIN roles r ON u.role_id = r.id WHERE u.id = ?1",
                    rusqlite::params![user_id],
                    |row| row.get(0),
                )
                .ok();
            match permissions_json {
                Some(json_str) => {
                    let perms: Vec<String> = serde_json::from_str(&json_str).unwrap_or_default();
                    Ok(perms)
                }
                None => Ok(vec![]),
            }
        })
        .await
        .context("Task join failed")?
    }

    /// Seed default roles if they don't exist yet.
    pub async fn seed_default_roles(&self) -> Result<()> {
        let existing = self.list_roles().await?;
        if existing.is_empty() {
            let now = crate::timeutil::now_secs();
            let _ = self.create_role("Admin", &serde_json::to_string(crate::models::ALL_PERMISSIONS).unwrap()).await;
            let _ = self.create_role("Operator", &serde_json::to_string(crate::models::OPERATOR_PERMISSIONS).unwrap()).await;
            let _ = self.create_role("Viewer", &serde_json::to_string(crate::models::VIEWER_PERMISSIONS).unwrap()).await;
        }
        Ok(())
    }

    // ── Electronic signatures ──

    pub async fn create_electronic_signature(
        &self,
        user_id: &str,
        action: &str,
        resource_type: &str,
        resource_id: &str,
        reason: &str,
        meaning: &str,
        signature_hash: &str,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let user_id = user_id.to_string();
        let action = action.to_string();
        let resource_type = resource_type.to_string();
        let resource_id = resource_id.to_string();
        let reason = reason.to_string();
        let meaning = meaning.to_string();
        let signature_hash = signature_hash.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO electronic_signatures (user_id, action, resource_type, resource_id, reason, meaning, signature_hash, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![user_id, action, resource_type, resource_id, reason, meaning, signature_hash, now],
            )
            .context("Insert electronic signature failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_electronic_signatures(
        &self,
        resource_type: Option<&str>,
        resource_id: Option<&str>,
    ) -> Result<Vec<crate::compliance::ElectronicSignature>> {
        let conn = self.conn.clone();
        let rt = resource_type.map(|s| s.to_string());
        let ri = resource_id.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let base_sql = "SELECT id, user_id, action, resource_type, resource_id, reason, meaning, signature_hash, created_at FROM electronic_signatures";
            let mut sql = String::from(base_sql);
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if rt.is_some() {
                sql.push_str(" WHERE resource_type = ?1");
                param_values.push(Box::new(rt.as_deref().unwrap()));
                if ri.is_some() {
                    sql.push_str(" AND resource_id = ?2");
                    param_values.push(Box::new(ri.as_deref().unwrap()));
                }
            }
            sql.push_str(" ORDER BY created_at DESC");

            let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|b| b.as_ref()).collect();

            let mut stmt = conn.prepare(&sql).context("Prepare failed")?;
            let rows = stmt.query_map(param_refs.as_slice(), |row| {
                Ok(crate::compliance::ElectronicSignature {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    action: row.get(2)?,
                    resource_type: row.get(3)?,
                    resource_id: row.get(4)?,
                    reason: row.get(5)?,
                    meaning: row.get(6)?,
                    signature_hash: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    // ── Compliance config ──

    pub async fn get_compliance_config(&self) -> Result<Option<crate::compliance::ComplianceConfig>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let config_json: Option<String> = conn
                .query_row(
                    "SELECT config FROM compliance_config WHERE id = 1",
                    [],
                    |row| row.get(0),
                )
                .ok();
            match config_json {
                Some(json_str) => {
                    let config: crate::compliance::ComplianceConfig = serde_json::from_str(&json_str)
                        .context("Failed to parse compliance config")?;
                    Ok(Some(config))
                }
                None => Ok(None),
            }
        })
        .await
        .context("Task join failed")?
    }

    pub async fn upsert_compliance_config(&self, config: &crate::compliance::ComplianceConfig) -> Result<()> {
        let conn = self.conn.clone();
        let config_json = serde_json::to_string(config).context("Failed to serialize compliance config")?;
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO compliance_config (id, config) VALUES (1, ?1) ON CONFLICT(id) DO UPDATE SET config = ?1",
                rusqlite::params![config_json],
            )
            .context("Upsert compliance config failed")?;
            Ok(())
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

        // Seed default RBAC roles if none exist
        if let Err(e) = self.seed_default_roles().await {
            tracing::warn!("Failed to seed default roles: {}", e);
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
            let priority = match severity.as_str() {
                "critical" => "critical",
                "info" => "info",
                _ => "warning",
            };
            conn.execute(
                "INSERT INTO alarms (source_key, severity, value, threshold, message, triggered_at, priority, state)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active')",
                rusqlite::params![source_key, severity, value, threshold, message, now, priority],
            )
            .context("Insert alarm failed")?;
            let id = conn.last_insert_rowid();
            // Create audit entry
            let _ = conn.execute(
                "INSERT INTO alarm_audit_log (alarm_id, action, performed_by, details, created_at) VALUES (?1, 'created', 'system', ?2, ?3)",
                rusqlite::params![id, format!("{{\"severity\":\"{}\",\"value\":{},\"threshold\":{}}}", severity, value, threshold), now],
            );
            Ok(id)
        })
        .await
        .context("Task join failed")?
    }

    /// Resolve all unresolved alarms for a source_key, applying deadband hysteresis.
    /// Only resolves alarms where `value < threshold - deadband`.
    /// Pass `None` for current_value to resolve unconditionally (no deadband check).
    /// Resolves regardless of acknowledgment status — once the value is back to
    /// normal, the alarm should clear so a new one can fire later.
    pub async fn resolve_active_alarms(&self, source_key: &str, current_value: Option<f64>) -> Result<()> {
        let conn = self.conn.clone();
        let source_key = source_key.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();

            // Build the query: resolve unresolved alarms where value < threshold - deadband.
            // When current_value is None, resolve all unresolved alarms (no deadband check).
            let (alarm_ids, details_json): (Vec<i64>, Vec<String>) = {
                let sql = match current_value {
                    Some(_) => {
                        "SELECT id, '{\"value\":' || ?1 || ', \"threshold\":' || threshold || ', \"deadband\":' || COALESCE(deadband, 0) || '}'
                         FROM alarms WHERE source_key = ?2 AND resolved_at IS NULL
                           AND ?1 < threshold - COALESCE(deadband, 0)"
                    }
                    None => {
                        "SELECT id, '{}' FROM alarms WHERE source_key = ?1 AND resolved_at IS NULL"
                    }
                };
                let mut stmt = conn.prepare(sql)?;
                let rows = match current_value {
                    Some(cv) => {
                        let rows = stmt.query_map(rusqlite::params![cv, source_key], |row| {
                            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                        })?;
                        rows.collect::<Result<Vec<_>, _>>().unwrap_or_default()
                    }
                    None => {
                        let rows = stmt.query_map(rusqlite::params![source_key], |row| {
                            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                        })?;
                        rows.collect::<Result<Vec<_>, _>>().unwrap_or_default()
                    }
                };
                rows.into_iter().unzip()
            };

            if alarm_ids.is_empty() {
                return Ok(());
            }

            // Resolve the matched alarms
            let placeholders: Vec<String> = alarm_ids.iter().map(|_| "?".to_string()).collect();
            let sql = format!(
                "UPDATE alarms SET resolved_at = ?, state = 'resolved' WHERE id IN ({})",
                placeholders.join(",")
            );
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];
            for id in &alarm_ids {
                params.push(Box::new(*id));
            }
            conn.execute(&sql, rusqlite::params_from_iter(params))
                .context("Resolve alarms failed")?;

            // Create audit entries for each resolved alarm
            for (i, alarm_id) in alarm_ids.iter().enumerate() {
                let details = details_json.get(i).map(|s| s.as_str()).unwrap_or("{}");
                let _ = conn.execute(
                    "INSERT INTO alarm_audit_log (alarm_id, action, performed_by, details, created_at) VALUES (?1, 'auto_resolved', 'system', ?2, ?3)",
                    rusqlite::params![alarm_id, details, now],
                );
            }

            Ok(())
        })
        .await
        .context("Task join failed")?
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

    /// Check if there is any unresolved alarm for this source_key+severity.
    /// Uses `resolved_at IS NULL` (regardless of acknowledgment status) so that
    /// a new alarm cannot fire until the previous one is fully resolved.
    /// This implements non-repeating alarm behavior.
    pub async fn has_unresolved_alarm(&self, source_key: &str, severity: &str) -> Result<bool> {
        let conn = self.conn.clone();
        let source_key = source_key.to_string();
        let severity = severity.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM alarms
                     WHERE source_key = ?1 AND severity = ?2
                       AND resolved_at IS NULL",
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
                    "SELECT id, source_key, COALESCE(source_name, ''), severity, COALESCE(priority, severity), value, threshold, message,
                            triggered_at, resolved_at, acknowledged_at, acknowledged_by, cooldown_until,
                            shelved_until, shelved_by, shelved_reason, COALESCE(deadband, 0), COALESCE(state, 'active')
                     FROM alarms
                     WHERE acknowledged_at IS NULL
                     ORDER BY triggered_at DESC LIMIT ?1",
                    vec![Box::new(limit)],
                ),
                Some(s) if s == "acknowledged" => (
                    "SELECT id, source_key, COALESCE(source_name, ''), severity, COALESCE(priority, severity), value, threshold, message,
                            triggered_at, resolved_at, acknowledged_at, acknowledged_by, cooldown_until,
                            shelved_until, shelved_by, shelved_reason, COALESCE(deadband, 0), COALESCE(state, 'active')
                     FROM alarms
                     WHERE acknowledged_at IS NOT NULL
                     ORDER BY triggered_at DESC LIMIT ?1",
                    vec![Box::new(limit)],
                ),
                _ => (
                    "SELECT id, source_key, COALESCE(source_name, ''), severity, COALESCE(priority, severity), value, threshold, message,
                            triggered_at, resolved_at, acknowledged_at, acknowledged_by, cooldown_until,
                            shelved_until, shelved_by, shelved_reason, COALESCE(deadband, 0), COALESCE(state, 'active')
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
                        source_name: row.get(2)?,
                        severity: row.get(3)?,
                        priority: row.get(4)?,
                        value: row.get(5)?,
                        threshold: row.get(6)?,
                        message: row.get(7)?,
                        triggered_at: row.get(8)?,
                        resolved_at: row.get(9)?,
                        acknowledged_at: row.get(10)?,
                        acknowledged_by: row.get(11)?,
                        cooldown_until: row.get(12)?,
                        shelved_until: row.get(13)?,
                        shelved_by: row.get(14)?,
                        shelved_reason: row.get(15)?,
                        deadband: row.get(16)?,
                        state: row.get(17)?,
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
                "UPDATE alarms SET acknowledged_at = ?1, acknowledged_by = ?2, cooldown_until = ?3, state = 'acknowledged' WHERE id = ?4",
                rusqlite::params![now, username, cooldown_until, id],
            )
            .context("Acknowledge alarm failed")?;
            // Create audit entry
            let _ = conn.execute(
                "INSERT INTO alarm_audit_log (alarm_id, action, performed_by, details, created_at) VALUES (?1, 'acknowledged', ?2, ?3, ?4)",
                rusqlite::params![id, username, format!("{{\"cooldown_secs\":{}}}", cooldown_secs), now],
            );
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn acknowledge_all(&self, username: &str, cooldown_secs: i64) -> Result<()> {
        let conn = self.conn.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            let cooldown_until = if cooldown_secs > 0 { Some(now + cooldown_secs) } else { None };

            // Find alarms that will be acknowledged, for audit entries
            let alarm_ids: Vec<i64> = {
                let mut stmt = conn.prepare(
                    "SELECT id FROM alarms WHERE acknowledged_at IS NULL",
                )?;
                let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
                rows.collect::<Result<Vec<_>, _>>().unwrap_or_default()
            };

            conn.execute(
                "UPDATE alarms SET acknowledged_at = ?1, acknowledged_by = ?2, cooldown_until = ?3, state = 'acknowledged'
                 WHERE acknowledged_at IS NULL",
                rusqlite::params![now, username, cooldown_until],
            )
            .context("Acknowledge all alarms failed")?;

            // Create audit entries for each acknowledged alarm
            for alarm_id in alarm_ids {
                let _ = conn.execute(
                    "INSERT INTO alarm_audit_log (alarm_id, action, performed_by, details, created_at) VALUES (?1, 'acknowledged', ?2, ?3, ?4)",
                    rusqlite::params![alarm_id, username, format!("{{\"cooldown_secs\":{}}}", cooldown_secs), now],
                );
            }

            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    // ── Alarm Management Overhaul methods ──

    pub async fn shelf_alarm(
        &self,
        id: i64,
        shelved_until: i64,
        shelved_by: &str,
        reason: &str,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let shelved_by = shelved_by.to_string();
        let reason = reason.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "UPDATE alarms SET shelved_until = ?1, shelved_by = ?2, shelved_reason = ?3, state = 'shelved' WHERE id = ?4",
                rusqlite::params![shelved_until, shelved_by, reason, id],
            )
            .context("Shelf alarm failed")?;
            // Create audit entry
            let _ = conn.execute(
                "INSERT INTO alarm_audit_log (alarm_id, action, performed_by, details, created_at) VALUES (?1, 'shelved', ?2, ?3, ?4)",
                rusqlite::params![id, shelved_by, format!("{{\"reason\":\"{}\",\"until\":{}}}", reason.replace('"', "\\\""), shelved_until), now],
            );
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn unshelf_alarm(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            // Determine new state based on acknowledgement status
            let new_state: String = conn
                .query_row(
                    "SELECT CASE WHEN acknowledged_at IS NOT NULL THEN 'acknowledged' ELSE 'active' END FROM alarms WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| "active".to_string());

            conn.execute(
                "UPDATE alarms SET shelved_until = NULL, shelved_by = NULL, shelved_reason = NULL, state = ?1 WHERE id = ?2",
                rusqlite::params![new_state, id],
            )
            .context("Unshelf alarm failed")?;
            // Create audit entry
            let _ = conn.execute(
                "INSERT INTO alarm_audit_log (alarm_id, action, performed_by, details, created_at) VALUES (?1, 'unshelved', 'system', '{}', ?2)",
                rusqlite::params![id, now],
            );
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn create_alarm_audit_entry(
        &self,
        alarm_id: i64,
        action: &str,
        performed_by: &str,
        details: &str,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let action = action.to_string();
        let performed_by = performed_by.to_string();
        let details = details.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO alarm_audit_log (alarm_id, action, performed_by, details, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![alarm_id, action, performed_by, details, now],
            )
            .context("Insert alarm audit entry failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_alarm_audit(
        &self,
        alarm_id: Option<i64>,
        from: Option<i64>,
        to: Option<i64>,
        limit: u32,
    ) -> Result<Vec<AlarmAuditEntry>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut sql = String::from(
                "SELECT id, alarm_id, action, performed_by, details, created_at FROM alarm_audit_log WHERE 1=1",
            );
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            if alarm_id.is_some() {
                sql.push_str(" AND alarm_id = ?");
                params.push(Box::new(alarm_id.unwrap()));
            }
            if from.is_some() {
                sql.push_str(" AND created_at >= ?");
                params.push(Box::new(from.unwrap()));
            }
            if to.is_some() {
                sql.push_str(" AND created_at <= ?");
                params.push(Box::new(to.unwrap()));
            }
            sql.push_str(" ORDER BY created_at DESC LIMIT ?");
            params.push(Box::new(limit));

            let mut stmt = conn.prepare(&sql).context("Prepare audit query failed")?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt
                .query_map(param_refs.as_slice(), |row| {
                    Ok(AlarmAuditEntry {
                        id: row.get(0)?,
                        alarm_id: row.get(1)?,
                        action: row.get(2)?,
                        performed_by: row.get(3)?,
                        details: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                })
                .context("Query audit failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_alarm_summary(&self) -> Result<AlarmSummary> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            let twenty_four_hours_ago = now - 86400;

            let active_critical: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM alarms WHERE state = 'active' AND priority = 'critical'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let active_warning: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM alarms WHERE state = 'active' AND priority = 'warning'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let active_info: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM alarms WHERE state = 'active' AND priority = 'info'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let shelved: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM alarms WHERE state = 'shelved'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let recent_24h: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM alarms WHERE triggered_at >= ?1",
                    rusqlite::params![twenty_four_hours_ago],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let avg_ack_time_secs: f64 = conn
                .query_row(
                    "SELECT AVG(acknowledged_at - triggered_at) FROM alarms WHERE acknowledged_at IS NOT NULL AND triggered_at >= ?1",
                    rusqlite::params![twenty_four_hours_ago],
                    |row| row.get::<_, Option<f64>>(0),
                )
                .ok()
                .flatten()
                .unwrap_or(0.0);

            Ok(AlarmSummary {
                active_critical,
                active_warning,
                active_info,
                shelved,
                recent_24h,
                avg_ack_time_secs,
            })
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_alarm_analytics(
        &self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Result<Vec<AlarmAnalyticsEntry>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut sql = String::from(
                "SELECT source_key, COALESCE(source_name, source_key) as source_name,
                        COUNT(*) as trigger_count,
                        COALESCE(AVG(CASE WHEN acknowledged_at IS NOT NULL THEN acknowledged_at - triggered_at END), 0) as avg_time_to_ack_secs,
                        MAX(triggered_at) as last_triggered_at
                 FROM alarms WHERE 1=1",
            );
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            if from.is_some() {
                sql.push_str(" AND triggered_at >= ?");
                params.push(Box::new(from.unwrap()));
            }
            if to.is_some() {
                sql.push_str(" AND triggered_at <= ?");
                params.push(Box::new(to.unwrap()));
            }
            sql.push_str(" GROUP BY source_key, source_name ORDER BY trigger_count DESC");

            let mut stmt = conn.prepare(&sql).context("Prepare analytics query failed")?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt
                .query_map(param_refs.as_slice(), |row| {
                    Ok(AlarmAnalyticsEntry {
                        source_key: row.get(0)?,
                        source_name: row.get(1)?,
                        trigger_count: row.get(2)?,
                        avg_time_to_ack_secs: row.get(3)?,
                        last_triggered_at: row.get(4)?,
                    })
                })
                .context("Query analytics failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
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
            created_at INTEGER NOT NULL,
            UNIQUE(device_id, node_id)
        );
        CREATE INDEX IF NOT EXISTS idx_opcua_nodes_device ON opcua_nodes(device_id);
        CREATE TABLE IF NOT EXISTS s7_devices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            rack INTEGER NOT NULL DEFAULT 0,
            slot INTEGER NOT NULL DEFAULT 1,
            interval_ms INTEGER NOT NULL DEFAULT 1000,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS s7_tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id INTEGER NOT NULL REFERENCES s7_devices(id) ON DELETE CASCADE,
            tag_address TEXT NOT NULL,
            name TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            data_type TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            UNIQUE(device_id, name)
        );
        CREATE INDEX IF NOT EXISTS idx_s7_tags_device ON s7_tags(device_id);
        CREATE TABLE IF NOT EXISTS enip_devices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            slot INTEGER NOT NULL DEFAULT 0,
            interval_ms INTEGER NOT NULL DEFAULT 1000,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS enip_tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id INTEGER NOT NULL REFERENCES enip_devices(id) ON DELETE CASCADE,
            tag_name TEXT NOT NULL,
            name TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            data_type TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            UNIQUE(device_id, tag_name)
        );
        CREATE INDEX IF NOT EXISTS idx_enip_tags_device ON enip_tags(device_id);
        CREATE TABLE IF NOT EXISTS hub_sites (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            api_key TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            last_seen INTEGER,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_hub_sites_enabled ON hub_sites(enabled);
        CREATE TABLE IF NOT EXISTS reports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            schedule TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            device_ids TEXT NOT NULL DEFAULT 'all',
            email_recipients TEXT NOT NULL,
            last_generated INTEGER,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS downtime_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id TEXT NOT NULL,
            state TEXT NOT NULL,
            started_at INTEGER NOT NULL,
            ended_at INTEGER,
            duration_secs INTEGER,
            reason TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_downtime_device ON downtime_events(device_id);
        CREATE INDEX IF NOT EXISTS idx_downtime_started ON downtime_events(started_at);
        CREATE TABLE IF NOT EXISTS shifts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            start_hour INTEGER NOT NULL,
            start_minute INTEGER NOT NULL,
            duration_hours INTEGER NOT NULL,
            days_mask INTEGER NOT NULL DEFAULT 127,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS roles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            permissions TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS electronic_signatures (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            action TEXT NOT NULL,
            resource_type TEXT NOT NULL,
            resource_id TEXT NOT NULL,
            reason TEXT NOT NULL,
            meaning TEXT NOT NULL,
            signature_hash TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_electronic_signatures_resource
            ON electronic_signatures(resource_type, resource_id);
        CREATE TABLE IF NOT EXISTS compliance_config (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            config TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS cameras (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            snapshot_on_alarm INTEGER NOT NULL DEFAULT 1,
            retention_hours INTEGER NOT NULL DEFAULT 72,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS camera_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            camera_id INTEGER NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
            timestamp INTEGER NOT NULL,
            alarm_source TEXT,
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            created_at INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_camera_snapshots_camera
            ON camera_snapshots(camera_id, timestamp DESC);
        CREATE TABLE IF NOT EXISTS annotations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tag_id TEXT NOT NULL,
            annotation_text TEXT NOT NULL,
            data_timestamp INTEGER NOT NULL,
            author_id INTEGER NOT NULL REFERENCES users(id),
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_annotations_tag_ts
            ON annotations(tag_id, data_timestamp);
        CREATE TABLE IF NOT EXISTS shift_handovers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            author_id INTEGER NOT NULL REFERENCES users(id),
            shift_date TEXT NOT NULL,
            shift_period TEXT NOT NULL CHECK(shift_period IN ('morning', 'afternoon', 'night')),
            what_happened TEXT NOT NULL DEFAULT '',
            whats_pending TEXT NOT NULL DEFAULT '',
            what_to_watch TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_shift_handovers_date_period
            ON shift_handovers(shift_date, shift_period);
        CREATE TABLE IF NOT EXISTS user_notifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL REFERENCES users(id),
            from_user_id INTEGER NOT NULL REFERENCES users(id),
            notification_type TEXT NOT NULL CHECK(notification_type IN ('mention', 'shift_handover')),
            message TEXT NOT NULL,
            reference_id INTEGER,
            read INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_user_notifications_user_read
            ON user_notifications(user_id, read, created_at);

        -- v0.26.0 -- Simulation profiles
        CREATE TABLE IF NOT EXISTS sim_profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            data_type TEXT NOT NULL,
            config TEXT NOT NULL DEFAULT '{}',
            created_at INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_sim_profiles_name ON sim_profiles(name);

        CREATE TABLE IF NOT EXISTS scripts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            source TEXT NOT NULL DEFAULT '',
            syntax_error TEXT,
            cpu_limit_ms INTEGER NOT NULL DEFAULT 1000,
            memory_limit_bytes INTEGER NOT NULL DEFAULT 1048576,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_scripts_name ON scripts(name);

        CREATE TABLE IF NOT EXISTS script_executions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            script_id INTEGER NOT NULL REFERENCES scripts(id) ON DELETE CASCADE,
            flow_id TEXT,
            trigger_type TEXT,
            duration_ms INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'success',
            error_message TEXT,
            output TEXT,
            created_at INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_script_executions_script ON script_executions(script_id);
        CREATE INDEX IF NOT EXISTS idx_script_executions_created ON script_executions(created_at);

        CREATE TABLE IF NOT EXISTS config_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            label TEXT,
            description TEXT,
            snapshot_data TEXT NOT NULL DEFAULT '{}',
            trigger TEXT NOT NULL DEFAULT 'auto',
            created_by TEXT,
            created_at INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_config_snapshots_created ON config_snapshots(created_at);
        CREATE INDEX IF NOT EXISTS idx_config_snapshots_trigger ON config_snapshots(trigger);",
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

    // Migration: add recipes table
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS recipes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            category TEXT,
            device_id TEXT NOT NULL,
            values TEXT NOT NULL,
            version INTEGER NOT NULL DEFAULT 1,
            description TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_recipes_device ON recipes(device_id);"
    );

    // Migration: add role_id column to users (roles table is created in main schema above)
    let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN role_id INTEGER REFERENCES roles(id)");

    // Migration: v0.22.0 — IEC 62443 security audit mode
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS security_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_type TEXT NOT NULL,
            severity TEXT NOT NULL DEFAULT 'info',
            source_ip TEXT,
            user_agent TEXT,
            username TEXT,
            details TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_security_log_type ON security_log(event_type);
        CREATE INDEX IF NOT EXISTS idx_security_log_created ON security_log(created_at);
        CREATE INDEX IF NOT EXISTS idx_security_log_severity ON security_log(severity);

        CREATE TABLE IF NOT EXISTS security_thresholds (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            failed_login_threshold INTEGER NOT NULL DEFAULT 5,
            failed_login_window_secs INTEGER NOT NULL DEFAULT 60,
            suspicious_401_threshold INTEGER NOT NULL DEFAULT 10,
            suspicious_401_window_secs INTEGER NOT NULL DEFAULT 60,
            rate_limit_threshold INTEGER NOT NULL DEFAULT 100,
            rate_limit_window_secs INTEGER NOT NULL DEFAULT 60
        );
        INSERT OR IGNORE INTO security_thresholds (id) VALUES (1);"
    );

        // Migration: v0.23.0 — Alarm Management Overhaul
        let _ = conn.execute_batch("ALTER TABLE alarms ADD COLUMN priority TEXT NOT NULL DEFAULT 'warning'");
        let _ = conn.execute_batch("ALTER TABLE alarms ADD COLUMN shelved_until INTEGER");
        let _ = conn.execute_batch("ALTER TABLE alarms ADD COLUMN shelved_by TEXT");
        let _ = conn.execute_batch("ALTER TABLE alarms ADD COLUMN shelved_reason TEXT");
        let _ = conn.execute_batch("ALTER TABLE alarms ADD COLUMN deadband REAL NOT NULL DEFAULT 0");
        let _ = conn.execute_batch("ALTER TABLE alarms ADD COLUMN state TEXT NOT NULL DEFAULT 'active'");
        let _ = conn.execute_batch("ALTER TABLE alarms ADD COLUMN source_name TEXT NOT NULL DEFAULT ''");

        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS alarm_audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                alarm_id INTEGER NOT NULL REFERENCES alarms(id) ON DELETE CASCADE,
                action TEXT NOT NULL,
                performed_by TEXT NOT NULL,
                details TEXT NOT NULL DEFAULT '',
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_alarm_audit_alarm_time
                ON alarm_audit_log(alarm_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_alarm_audit_action_time
                ON alarm_audit_log(action, created_at);"
        );

        // Migration: v0.24.0 — Command & Control tables
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS commands (
                id TEXT PRIMARY KEY,
                data_source_id INTEGER NOT NULL,
                device_id INTEGER NOT NULL,
                protocol TEXT NOT NULL,
                value TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                policy TEXT NOT NULL DEFAULT 'immediate',
                scheduled_at INTEGER,
                retry_count INTEGER NOT NULL DEFAULT 0,
                max_retries INTEGER NOT NULL DEFAULT 3,
                error_message TEXT,
                created_at INTEGER NOT NULL,
                sent_at INTEGER,
                acked_at INTEGER,
                resolved_at INTEGER,
                created_by TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_commands_status ON commands(status);
            CREATE INDEX IF NOT EXISTS idx_commands_data_source ON commands(data_source_id);
            CREATE INDEX IF NOT EXISTS idx_commands_device ON commands(device_id);
            CREATE INDEX IF NOT EXISTS idx_commands_created ON commands(created_at);

            CREATE TABLE IF NOT EXISTS command_audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                command_id TEXT NOT NULL REFERENCES commands(id) ON DELETE CASCADE,
                old_status TEXT NOT NULL,
                new_status TEXT NOT NULL,
                changed_by TEXT NOT NULL,
                reason TEXT,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_command_audit_command ON command_audit_log(command_id);"
        );

        // Migration: v0.24.0 — Write policy columns on data_sources
        let _ = conn.execute_batch("ALTER TABLE data_sources ADD COLUMN writable INTEGER NOT NULL DEFAULT 0");
        let _ = conn.execute_batch("ALTER TABLE data_sources ADD COLUMN write_min REAL");
        let _ = conn.execute_batch("ALTER TABLE data_sources ADD COLUMN write_max REAL");
        let _ = conn.execute_batch("ALTER TABLE data_sources ADD COLUMN write_max_rate INTEGER");
        let _ = conn.execute_batch("ALTER TABLE data_sources ADD COLUMN write_requires_reason INTEGER NOT NULL DEFAULT 0");
        let _ = conn.execute_batch("ALTER TABLE data_sources ADD COLUMN write_deadband REAL");

        // Migration: v0.26.0 — Simulation columns on devices
        let _ = conn.execute_batch("ALTER TABLE devices ADD COLUMN simulation_mode INTEGER NOT NULL DEFAULT 0");
        let _ = conn.execute_batch("ALTER TABLE devices ADD COLUMN sim_profile_id INTEGER");
        let _ = conn.execute_batch("ALTER TABLE devices ADD COLUMN sim_config TEXT");
        // Migration: v0.26.0 — Simulation column on data_sources
        let _ = conn.execute_batch("ALTER TABLE data_sources ADD COLUMN sim_value REAL");

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

    pub async fn count_modbus_devices(&self) -> Result<u32> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM modbus_devices",
                [],
                |row| row.get(0),
            ).context("Count query failed")?;
            Ok(count as u32)
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

    pub async fn get_opcua_device(&self, id: i64) -> Result<Option<OpcUaDevice>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let result = conn.query_row(
                "SELECT id, name, url, security_policy, security_mode, username, password, interval_ms, enabled, created_at FROM opcua_devices WHERE id = ?1",
                rusqlite::params![id],
                |row| {
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
                },
            );
            match result {
                Ok(device) => Ok(Some(device)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e).context("Get opcua device failed"),
            }
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

            // Look up device name and load all nodes to clean up data sources
            let device_name: Option<String> = conn
                .query_row(
                    "SELECT name FROM opcua_devices WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .ok();

            if let Some(device_name) = &device_name {
                let mut stmt = conn
                    .prepare("SELECT name FROM opcua_nodes WHERE device_id = ?1")
                    .context("Prepare select opcua nodes failed")?;

                let node_rows = stmt
                    .query_map(rusqlite::params![id], |row| row.get::<_, String>(0))
                    .context("Query opcua nodes failed")?;

                for node_row in node_rows {
                    if let Ok(node_name) = node_row {
                        let source_key = format!("{}:{}", device_name, node_name);
                        let _ = conn.execute(
                            "DELETE FROM data_sources WHERE key = ?1",
                            rusqlite::params![source_key],
                        );
                    }
                }
            }

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
        device_name: &str,
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
        let device_name = device_name.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO opcua_nodes (device_id, node_id, name, display_name, data_type, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![device_id, node_id, name, display_name, data_type, now],
            )
            .context("Insert opcua node failed")?;

            // Auto-create data source for this node
            let source_key = format!("{}:{}", device_name, name);
            let ds_name = if display_name.is_empty() {
                name.clone()
            } else {
                display_name.clone()
            };
            let _ = conn.execute(
                "INSERT OR IGNORE INTO data_sources (key, name, unit, color, min_val, max_val, warn_val, crit_val,
                 base_val, sort_order, decimals, source_type, source_config, device_id, chart_type)
                 VALUES (?1, ?2, '', '#f97316', 0, 0, 0, 0, 0, 0, 1, 'opcua', '{}', ?3, 'sparkline')",
                rusqlite::params![source_key, ds_name, device_name],
            );

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

            // Look up node name and device name to delete corresponding data source
            if let Ok((node_name, device_id)) = conn.query_row(
                "SELECT name, device_id FROM opcua_nodes WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            ) {
                if let Ok(device_name) = conn.query_row(
                    "SELECT name FROM opcua_devices WHERE id = ?1",
                    rusqlite::params![device_id],
                    |row| row.get::<_, String>(0),
                ) {
                    let source_key = format!("{}:{}", device_name, node_name);
                    let _ = conn.execute(
                        "DELETE FROM data_sources WHERE key = ?1",
                        rusqlite::params![source_key],
                    );
                }
            }

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

    // ── EtherNet/IP device management ──

    pub async fn create_enip_device(
        &self,
        name: &str,
        url: &str,
        slot: u8,
        interval_ms: u64,
        enabled: bool,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let url = url.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO enip_devices (name, url, slot, interval_ms, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![name, url, slot, interval_ms, enabled as i64, now],
            )
            .context("Insert enip device failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_enip_device(&self, id: i64) -> Result<Option<EnipDevice>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let result = conn.query_row(
                "SELECT id, name, url, slot, interval_ms, enabled, created_at FROM enip_devices WHERE id = ?1",
                rusqlite::params![id],
                |row| {
                    Ok(EnipDevice {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        url: row.get(2)?,
                        slot: row.get(3)?,
                        interval_ms: row.get(4)?,
                        enabled: row.get::<_, i64>(5)? != 0,
                        created_at: row.get(6)?,
                    })
                },
            );
            match result {
                Ok(device) => Ok(Some(device)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e).context("Get enip device failed"),
            }
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_enip_devices(&self) -> Result<Vec<EnipDevice>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, url, slot, interval_ms, enabled, created_at FROM enip_devices ORDER BY created_at"
                )
                .context("Prepare list_enip_devices failed")?;

            let rows = stmt.query_map([], |row| {
                Ok(EnipDevice {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    slot: row.get(3)?,
                    interval_ms: row.get(4)?,
                    enabled: row.get::<_, i64>(5)? != 0,
                    created_at: row.get(6)?,
                })
            })
            .context("Query list_enip_devices failed")?;

            let mut devices = Vec::new();
            for row in rows {
                devices.push(row.context("Row list_enip_devices failed")?);
            }
            Ok(devices)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_enip_device(
        &self,
        id: i64,
        name: Option<&str>,
        url: Option<&str>,
        slot: Option<u8>,
        interval_ms: Option<u64>,
        enabled: Option<bool>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let name = name.map(|s| s.to_string());
        let url = url.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            if let Some(name) = &name {
                conn.execute("UPDATE enip_devices SET name = ?1 WHERE id = ?2", rusqlite::params![name, id])
                    .context("Update enip device name failed")?;
            }
            if let Some(url) = &url {
                conn.execute("UPDATE enip_devices SET url = ?1 WHERE id = ?2", rusqlite::params![url, id])
                    .context("Update enip device url failed")?;
            }
            if let Some(slot) = slot {
                conn.execute("UPDATE enip_devices SET slot = ?1 WHERE id = ?2", rusqlite::params![slot, id])
                    .context("Update enip device slot failed")?;
            }
            if let Some(interval_ms) = interval_ms {
                conn.execute("UPDATE enip_devices SET interval_ms = ?1 WHERE id = ?2", rusqlite::params![interval_ms, id])
                    .context("Update enip device interval_ms failed")?;
            }
            if let Some(enabled) = enabled {
                conn.execute("UPDATE enip_devices SET enabled = ?1 WHERE id = ?2", rusqlite::params![enabled as i64, id])
                    .context("Update enip device enabled failed")?;
            }
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_enip_device(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            // Look up device name and load all tags to clean up data sources
            let device_name: Option<String> = conn
                .query_row(
                    "SELECT name FROM enip_devices WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .ok();

            if let Some(device_name) = &device_name {
                let mut stmt = conn
                    .prepare("SELECT name FROM enip_tags WHERE device_id = ?1")
                    .context("Prepare select enip tags failed")?;

                let tag_rows = stmt
                    .query_map(rusqlite::params![id], |row| row.get::<_, String>(0))
                    .context("Query enip tags failed")?;

                for tag_row in tag_rows {
                    if let Ok(tag_name) = tag_row {
                        let source_key = format!("{}:{}", device_name, tag_name);
                        let _ = conn.execute(
                            "DELETE FROM data_sources WHERE key = ?1",
                            rusqlite::params![source_key],
                        );
                    }
                }
            }

            conn.execute("DELETE FROM enip_devices WHERE id = ?1", rusqlite::params![id])
                .context("Delete enip device failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }

    pub async fn create_enip_tag(
        &self,
        device_id: i64,
        device_name: &str,
        tag_name: &str,
        name: &str,
        display_name: &str,
        data_type: &str,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let tag_name = tag_name.to_string();
        let name = name.to_string();
        let display_name = display_name.to_string();
        let data_type = data_type.to_string();
        let device_name = device_name.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO enip_tags (device_id, tag_name, name, display_name, data_type, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![device_id, tag_name, name, display_name, data_type, now],
            )
            .context("Insert enip tag failed")?;

            // Auto-create data source for this tag
            let source_key = format!("{}:{}", device_name, name);
            let ds_name = if display_name.is_empty() {
                name.clone()
            } else {
                display_name.clone()
            };
            let _ = conn.execute(
                "INSERT OR IGNORE INTO data_sources (key, name, unit, color, min_val, max_val, warn_val, crit_val,
                 base_val, sort_order, decimals, source_type, source_config, device_id, chart_type)
                 VALUES (?1, ?2, '', '#f97316', 0, 0, 0, 0, 0, 0, 1, 'enip', '{}', ?3, 'sparkline')",
                rusqlite::params![source_key, ds_name, device_name],
            );

            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_enip_tags(&self, device_id: i64) -> Result<Vec<EnipTag>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, device_id, tag_name, name, display_name, data_type, created_at FROM enip_tags WHERE device_id = ?1 ORDER BY created_at"
                )
                .context("Prepare list_enip_tags failed")?;

            let rows = stmt.query_map(rusqlite::params![device_id], |row| {
                Ok(EnipTag {
                    id: row.get(0)?,
                    device_id: row.get(1)?,
                    tag_name: row.get(2)?,
                    name: row.get(3)?,
                    display_name: row.get(4)?,
                    data_type: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .context("Query list_enip_tags failed")?;

            let mut tags = Vec::new();
            for row in rows {
                tags.push(row.context("Row list_enip_tags failed")?);
            }
            Ok(tags)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_enip_tag(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            // Look up tag name and device name to delete corresponding data source
            if let Ok((tag_name, device_id)) = conn.query_row(
                "SELECT name, device_id FROM enip_tags WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            ) {
                if let Ok(device_name) = conn.query_row(
                    "SELECT name FROM enip_devices WHERE id = ?1",
                    rusqlite::params![device_id],
                    |row| row.get::<_, String>(0),
                ) {
                    let source_key = format!("{}:{}", device_name, tag_name);
                    let _ = conn.execute(
                        "DELETE FROM data_sources WHERE key = ?1",
                        rusqlite::params![source_key],
                    );
                }
            }

            conn.execute("DELETE FROM enip_tags WHERE id = ?1", rusqlite::params![id])
                .context("Delete enip tag failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }

    /// Load all enabled EtherNet/IP devices with their tags and convert to runtime configs.
    pub async fn get_all_enip_device_configs(&self) -> Result<Vec<EnipDeviceConfig>> {
        let devices = self.list_enip_devices().await?;
        let mut configs = Vec::new();
        for device in &devices {
            if !device.enabled { continue; }
            let tags = self.list_enip_tags(device.id).await?;
            let device_key = format!("enip_{}", device.id);
            configs.push(EnipDeviceConfig {
                id: device_key,
                url: device.url.clone(),
                slot: device.slot,
                interval_ms: device.interval_ms,
                tags: tags.iter().map(|t| crate::ethernet_ip::EnipTagConfig {
                    tag_name: t.tag_name.clone(),
                    name: t.name.clone(),
                    data_type: t.data_type.clone(),
                }).collect(),
            });
        }
        Ok(configs)
    }
    // ── S7 (Siemens) device management ──

    pub async fn create_s7_device(
        &self,
        name: &str,
        url: &str,
        rack: u8,
        slot: u8,
        interval_ms: u64,
        enabled: bool,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let url = url.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO s7_devices (name, url, rack, slot, interval_ms, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![name, url, rack, slot, interval_ms, enabled as i64, now],
            )
            .context("Insert s7 device failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_s7_devices(&self) -> Result<Vec<S7Device>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, url, rack, slot, interval_ms, enabled, created_at FROM s7_devices ORDER BY created_at"
                )
                .context("Prepare list_s7_devices failed")?;

            let rows = stmt.query_map([], |row| {
                Ok(S7Device {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    rack: row.get(3)?,
                    slot: row.get(4)?,
                    interval_ms: row.get(5)?,
                    enabled: row.get::<_, i64>(6)? != 0,
                    created_at: row.get(7)?,
                })
            })
            .context("Query list_s7_devices failed")?;

            let mut devices = Vec::new();
            for row in rows {
                devices.push(row.context("Row list_s7_devices failed")?);
            }
            Ok(devices)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_s7_device(&self, id: i64) -> Result<Option<S7Device>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let result = conn.query_row(
                "SELECT id, name, url, rack, slot, interval_ms, enabled, created_at FROM s7_devices WHERE id = ?1",
                rusqlite::params![id],
                |row| {
                    Ok(S7Device {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        url: row.get(2)?,
                        rack: row.get(3)?,
                        slot: row.get(4)?,
                        interval_ms: row.get(5)?,
                        enabled: row.get::<_, i64>(6)? != 0,
                        created_at: row.get(7)?,
                    })
                },
            );
            match result {
                Ok(device) => Ok(Some(device)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e).context("Get s7 device failed"),
            }
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_s7_device(
        &self,
        id: i64,
        name: Option<&str>,
        url: Option<&str>,
        rack: Option<u8>,
        slot: Option<u8>,
        interval_ms: Option<u64>,
        enabled: Option<bool>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let name = name.map(|s| s.to_string());
        let url = url.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            if let Some(name) = &name {
                conn.execute("UPDATE s7_devices SET name = ?1 WHERE id = ?2", rusqlite::params![name, id])
                    .context("Update s7 device name failed")?;
            }
            if let Some(url) = &url {
                conn.execute("UPDATE s7_devices SET url = ?1 WHERE id = ?2", rusqlite::params![url, id])
                    .context("Update s7 device url failed")?;
            }
            if let Some(rack) = rack {
                conn.execute("UPDATE s7_devices SET rack = ?1 WHERE id = ?2", rusqlite::params![rack, id])
                    .context("Update s7 device rack failed")?;
            }
            if let Some(slot) = slot {
                conn.execute("UPDATE s7_devices SET slot = ?1 WHERE id = ?2", rusqlite::params![slot, id])
                    .context("Update s7 device slot failed")?;
            }
            if let Some(interval_ms) = interval_ms {
                conn.execute("UPDATE s7_devices SET interval_ms = ?1 WHERE id = ?2", rusqlite::params![interval_ms, id])
                    .context("Update s7 device interval_ms failed")?;
            }
            if let Some(enabled) = enabled {
                conn.execute("UPDATE s7_devices SET enabled = ?1 WHERE id = ?2", rusqlite::params![enabled as i64, id])
                    .context("Update s7 device enabled failed")?;
            }
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_s7_device(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let device_name: Option<String> = conn
                .query_row(
                    "SELECT name FROM s7_devices WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .ok();

            if let Some(device_name) = &device_name {
                let mut stmt = conn
                    .prepare("SELECT name FROM s7_tags WHERE device_id = ?1")
                    .context("Prepare select s7 tags failed")?;

                let tag_rows = stmt
                    .query_map(rusqlite::params![id], |row| row.get::<_, String>(0))
                    .context("Query s7 tags failed")?;

                for tag_row in tag_rows {
                    if let Ok(tag_name) = tag_row {
                        let source_key = format!("s7_{}:{}", device_name, tag_name);
                        let _ = conn.execute(
                            "DELETE FROM data_sources WHERE key = ?1",
                            rusqlite::params![source_key],
                        );
                    }
                }
            }

            conn.execute("DELETE FROM s7_devices WHERE id = ?1", rusqlite::params![id])
                .context("Delete s7 device failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }

    pub async fn create_s7_tag(
        &self,
        device_id: i64,
        device_name: &str,
        tag_address: &str,
        name: &str,
        display_name: &str,
        data_type: &str,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let tag_address = tag_address.to_string();
        let name = name.to_string();
        let display_name = display_name.to_string();
        let data_type = data_type.to_string();
        let device_name = device_name.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO s7_tags (device_id, tag_address, name, display_name, data_type, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![device_id, tag_address, name, display_name, data_type, now],
            )
            .context("Insert s7 tag failed")?;

            let source_key = format!("s7_{}:{}", device_name, name);
            let ds_name = if display_name.is_empty() {
                name.clone()
            } else {
                display_name.clone()
            };
            let _ = conn.execute(
                "INSERT OR IGNORE INTO data_sources (key, name, unit, color, min_val, max_val, warn_val, crit_val,
                 base_val, sort_order, decimals, source_type, source_config, device_id, chart_type)
                 VALUES (?1, ?2, '', '#3b82f6', 0, 0, 0, 0, 0, 0, 1, 's7', '{}', ?3, 'sparkline')",
                rusqlite::params![source_key, ds_name, device_name],
            );

            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_s7_tags(&self, device_id: i64) -> Result<Vec<S7TagRow>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, device_id, tag_address, name, display_name, data_type, created_at FROM s7_tags WHERE device_id = ?1 ORDER BY created_at"
                )
                .context("Prepare list_s7_tags failed")?;

            let rows = stmt.query_map(rusqlite::params![device_id], |row| {
                Ok(S7TagRow {
                    id: row.get(0)?,
                    device_id: row.get(1)?,
                    tag_address: row.get(2)?,
                    name: row.get(3)?,
                    display_name: row.get(4)?,
                    data_type: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .context("Query list_s7_tags failed")?;

            let mut tags = Vec::new();
            for row in rows {
                tags.push(row.context("Row list_s7_tags failed")?);
            }
            Ok(tags)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_s7_tag(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            if let Ok((tag_name, device_id)) = conn.query_row(
                "SELECT name, device_id FROM s7_tags WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            ) {
                if let Ok(device_name) = conn.query_row(
                    "SELECT name FROM s7_devices WHERE id = ?1",
                    rusqlite::params![device_id],
                    |row| row.get::<_, String>(0),
                ) {
                    let source_key = format!("s7_{}:{}", device_name, tag_name);
                    let _ = conn.execute(
                        "DELETE FROM data_sources WHERE key = ?1",
                        rusqlite::params![source_key],
                    );
                }
            }

            conn.execute("DELETE FROM s7_tags WHERE id = ?1", rusqlite::params![id])
                .context("Delete s7 tag failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }

    pub async fn get_all_s7_device_configs(&self) -> Result<Vec<crate::s7::S7DeviceConfig>> {
        let devices = self.list_s7_devices().await?;
        let mut configs = Vec::new();
        for device in &devices {
            if !device.enabled { continue; }
            let tags = self.list_s7_tags(device.id).await?;
            let device_key = format!("s7_{}", device.id);
            let mut s7_tags = Vec::new();
            for tag_row in &tags {
                match crate::s7::S7Client::parse_tag_address(&tag_row.tag_address) {
                    Ok(s7_tag) => s7_tags.push(s7_tag),
                    Err(e) => {
                        tracing::warn!(
                            "Failed to parse S7 tag '{}' for device '{}': {}",
                            tag_row.tag_address, device.name, e
                        );
                    }
                }
            }
            configs.push(crate::s7::S7DeviceConfig {
                id: device_key,
                url: device.url.clone(),
                rack: device.rack,
                slot: device.slot,
                interval_ms: device.interval_ms,
                tags: s7_tags,
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

// ── Hub Sites management ──

impl Storage {
    pub async fn create_hub_site(&self, name: &str, url: &str, api_key: &str, enabled: bool) -> Result<i64> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let url = url.to_string();
        let api_key = api_key.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO hub_sites (name, url, api_key, enabled, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![name, url, api_key, enabled as i64, now],
            )
            .context("Insert hub_site failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_hub_sites(&self) -> Result<Vec<HubSite>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, url, api_key, enabled, last_seen, created_at FROM hub_sites ORDER BY created_at"
                )
                .context("Prepare list_hub_sites failed")?;
            let rows = stmt.query_map([], |row| {
                Ok(HubSite {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    api_key: row.get(3)?,
                    enabled: row.get::<_, i64>(4)? != 0,
                    last_seen: row.get(5)?,
                    created_at: row.get(6)?,
                })
            }).context("Query list_hub_sites failed")?;
            rows.collect::<Result<Vec<_>, _>>().context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_hub_site(&self, id: i64, name: Option<&str>, url: Option<&str>, api_key: Option<&str>, enabled: Option<bool>) -> Result<()> {
        let conn = self.conn.clone();
        let name = name.map(|s| s.to_string());
        let url = url.map(|s| s.to_string());
        let api_key = api_key.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            if let Some(name) = &name {
                conn.execute("UPDATE hub_sites SET name = ?1 WHERE id = ?2", rusqlite::params![name, id])
                    .context("Update hub_site name failed")?;
            }
            if let Some(url) = &url {
                conn.execute("UPDATE hub_sites SET url = ?1 WHERE id = ?2", rusqlite::params![url, id])
                    .context("Update hub_site url failed")?;
            }
            if let Some(api_key) = &api_key {
                conn.execute("UPDATE hub_sites SET api_key = ?1 WHERE id = ?2", rusqlite::params![api_key, id])
                    .context("Update hub_site api_key failed")?;
            }
            if let Some(enabled) = enabled {
                conn.execute("UPDATE hub_sites SET enabled = ?1 WHERE id = ?2", rusqlite::params![enabled as i64, id])
                    .context("Update hub_site enabled failed")?;
            }
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_hub_site(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM hub_sites WHERE id = ?1", rusqlite::params![id])
                .context("Delete hub_site failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_hub_site_last_seen(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute("UPDATE hub_sites SET last_seen = ?1 WHERE id = ?2", rusqlite::params![now, id])
                .context("Update hub_site last_seen failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_enabled_hub_sites(&self) -> Result<Vec<HubSite>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, url, api_key, enabled, last_seen, created_at FROM hub_sites WHERE enabled = 1 ORDER BY created_at"
                )
                .context("Prepare get_enabled_hub_sites failed")?;
            let rows = stmt.query_map([], |row| {
                Ok(HubSite {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    api_key: row.get(3)?,
                    enabled: row.get::<_, i64>(4)? != 0,
                    last_seen: row.get(5)?,
                    created_at: row.get(6)?,
                })
            }).context("Query get_enabled_hub_sites failed")?;
            rows.collect::<Result<Vec<_>, _>>().context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
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

    // ── Report management ──

    pub async fn list_reports(&self) -> Result<Vec<Report>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, name, schedule, enabled, device_ids, email_recipients, last_generated, created_at FROM reports ORDER BY id"
            ).context("Prepare list_reports failed")?;
            let rows = stmt.query_map([], |row| {
                Ok(Report {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    schedule: row.get(2)?,
                    enabled: row.get::<_, i64>(3)? != 0,
                    device_ids: row.get(4)?,
                    email_recipients: row.get(5)?,
                    last_generated: row.get(6)?,
                    created_at: row.get(7)?,
                })
            }).context("Query list_reports failed")?;
            let mut reports = Vec::new();
            for row in rows {
                reports.push(row.context("Row parse list_reports failed")?);
            }
            Ok(reports)
        }).await.context("Task join failed")?
    }

    pub async fn get_report(&self, id: i64) -> Result<Option<Report>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, name, schedule, enabled, device_ids, email_recipients, last_generated, created_at FROM reports WHERE id = ?1"
            ).context("Prepare get_report failed")?;
            let result = stmt.query_row(rusqlite::params![id], |row| {
                Ok(Report {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    schedule: row.get(2)?,
                    enabled: row.get::<_, i64>(3)? != 0,
                    device_ids: row.get(4)?,
                    email_recipients: row.get(5)?,
                    last_generated: row.get(6)?,
                    created_at: row.get(7)?,
                })
            }).ok();
            Ok(result)
        }).await.context("Task join failed")?
    }

    pub async fn create_report(
        &self,
        name: &str,
        schedule: &str,
        enabled: bool,
        device_ids: &str,
        email_recipients: &str,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let schedule = schedule.to_string();
        let device_ids = device_ids.to_string();
        let email_recipients = email_recipients.to_string();
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO reports (name, schedule, enabled, device_ids, email_recipients, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![name, schedule, enabled as i64, device_ids, email_recipients, now],
            ).context("Insert report failed")?;
            Ok(conn.last_insert_rowid())
        }).await.context("Task join failed")?
    }

    pub async fn update_report(
        &self,
        id: i64,
        name: Option<&str>,
        schedule: Option<&str>,
        device_ids: Option<&str>,
        email_recipients: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let name = name.map(|s| s.to_string());
        let schedule = schedule.map(|s| s.to_string());
        let device_ids = device_ids.map(|s| s.to_string());
        let email_recipients = email_recipients.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            if let Some(name) = name {
                conn.execute("UPDATE reports SET name = ?1 WHERE id = ?2", rusqlite::params![name, id])
                    .context("Update report name failed")?;
            }
            if let Some(schedule) = schedule {
                conn.execute("UPDATE reports SET schedule = ?1 WHERE id = ?2", rusqlite::params![schedule, id])
                    .context("Update report schedule failed")?;
            }
            if let Some(device_ids) = device_ids {
                conn.execute("UPDATE reports SET device_ids = ?1 WHERE id = ?2", rusqlite::params![device_ids, id])
                    .context("Update report device_ids failed")?;
            }
            if let Some(email_recipients) = email_recipients {
                conn.execute("UPDATE reports SET email_recipients = ?1 WHERE id = ?2", rusqlite::params![email_recipients, id])
                    .context("Update report email_recipients failed")?;
            }
            if let Some(enabled) = enabled {
                conn.execute("UPDATE reports SET enabled = ?1 WHERE id = ?2", rusqlite::params![enabled as i64, id])
                    .context("Update report enabled failed")?;
            }
            Ok(())
        }).await.context("Task join failed")?
    }

    pub async fn delete_report(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM reports WHERE id = ?1", rusqlite::params![id])
                .context("Delete report failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    pub async fn update_report_last_generated(&self, id: i64, ts: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE reports SET last_generated = ?1 WHERE id = ?2",
                rusqlite::params![ts, id],
            ).context("Update report last_generated failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    // ── OEE / Downtime methods ──

    /// Create an open downtime event.
    pub async fn create_downtime_event(&self, device_id: &str, state: &str) -> Result<i64> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        let state = state.to_string();
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO downtime_events (device_id, state, started_at, created_at) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![device_id, state, now, now],
            ).context("Insert downtime event failed")?;
            Ok(conn.last_insert_rowid())
        }).await.context("Task join failed")?
    }

    /// Close a downtime event by setting ended_at and calculating duration.
    pub async fn close_downtime_event(&self, id: i64, reason: Option<&str>) -> Result<()> {
        let conn = self.conn.clone();
        let reason = reason.map(|s| s.to_string());
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            // Calculate duration from started_at
            let started_at: i64 = conn.query_row(
                "SELECT started_at FROM downtime_events WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            ).context("Failed to get downtime started_at")?;
            let duration = now - started_at;
            conn.execute(
                "UPDATE downtime_events SET ended_at = ?1, duration_secs = ?2, reason = ?3 WHERE id = ?4 AND ended_at IS NULL",
                rusqlite::params![now, duration, reason, id],
            ).context("Close downtime event failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    /// List downtime events with optional filters.
    pub async fn list_downtime_events(
        &self,
        device_id: Option<&str>,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Result<Vec<DowntimeEvent>> {
        let conn = self.conn.clone();
        let device_id = device_id.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut sql = String::from("SELECT id, device_id, state, started_at, ended_at, duration_secs, reason FROM downtime_events WHERE 1=1");
            if device_id.is_some() {
                sql.push_str(&format!(" AND device_id = '{}'", device_id.as_ref().unwrap().replace('\'', "''")));
            }
            if let Some(from) = from {
                sql.push_str(&format!(" AND started_at >= {}", from));
            }
            if let Some(to) = to {
                sql.push_str(&format!(" AND started_at <= {}", to));
            }
            sql.push_str(" ORDER BY started_at DESC");

            let mut stmt = conn.prepare(&sql).context("Prepare list downtime events failed")?;
            let rows = stmt.query_map([], |row| {
                Ok(DowntimeEvent {
                    id: row.get(0)?,
                    device_id: row.get(1)?,
                    state: row.get(2)?,
                    started_at: row.get(3)?,
                    ended_at: row.get(4)?,
                    duration_secs: row.get(5)?,
                    reason: row.get(6)?,
                })
            }).context("Query list downtime events failed")?;

            let mut events = Vec::new();
            for row in rows {
                events.push(row.context("Row parse downtime event failed")?);
            }
            Ok(events)
        }).await.context("Task join failed")?
    }

    /// Get the currently open (unclosed) downtime event for a device.
    pub async fn get_open_downtime_event(&self, device_id: &str) -> Result<Option<DowntimeEvent>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, device_id, state, started_at, ended_at, duration_secs, reason FROM downtime_events WHERE device_id = ?1 AND ended_at IS NULL ORDER BY started_at DESC LIMIT 1"
            ).context("Prepare get open downtime event failed")?;
            let mut rows = stmt.query_map(rusqlite::params![device_id], |row| {
                Ok(DowntimeEvent {
                    id: row.get(0)?,
                    device_id: row.get(1)?,
                    state: row.get(2)?,
                    started_at: row.get(3)?,
                    ended_at: row.get(4)?,
                    duration_secs: row.get(5)?,
                    reason: row.get(6)?,
                })
            }).context("Query get open downtime event failed")?;
            match rows.next() {
                Some(row) => Ok(Some(row.context("Row parse open downtime event failed")?)),
                None => Ok(None),
            }
        }).await.context("Task join failed")?
    }

    /// Calculate OEE metrics for a device over a time range.
    /// Availability = total_running_secs / total_planned_secs
    /// Performance and Quality default to 1.0 (to be enriched by user configuration).
    /// OEE = Availability * Performance * Quality
    pub async fn calculate_oee(
        &self,
        device_id: &str,
        from: i64,
        to: i64,
    ) -> Result<OeeMetrics> {
        let events = self.list_downtime_events(Some(device_id), Some(from), Some(to)).await?;

        let total_planned_secs = if to > from { to - from } else { 0 };

        // Total downtime = sum of all downtime durations (including currently open events)
        let mut total_downtime_secs: i64 = 0;
        let now = crate::timeutil::now_secs();
        let mut downtime_count: i64 = 0;

        for event in &events {
            downtime_count += 1;
            if let Some(duration) = event.duration_secs {
                total_downtime_secs += duration;
            } else if event.ended_at.is_none() {
                // Still open — calculate from started_at to now
                total_downtime_secs += (now - event.started_at).max(0);
            }
        }

        let total_running_secs = (total_planned_secs - total_downtime_secs).max(0);

        let availability = if total_planned_secs > 0 {
            total_running_secs as f64 / total_planned_secs as f64
        } else {
            1.0
        };

        // Performance and quality default to 1.0 until enriched with production data
        let performance = 1.0;
        let quality = 1.0;
        let oee = availability * performance * quality;

        Ok(OeeMetrics {
            availability,
            performance,
            quality,
            oee,
            total_running_secs,
            total_planned_secs,
            downtime_count,
        })
    }

    // ── Shift methods ──

    /// Create a shift configuration.
    pub async fn create_shift(
        &self,
        name: &str,
        start_hour: u32,
        start_minute: u32,
        duration_hours: u32,
        days_mask: u32,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO shifts (name, start_hour, start_minute, duration_hours, days_mask, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![name, start_hour, start_minute, duration_hours, days_mask, now],
            ).context("Insert shift failed")?;
            Ok(conn.last_insert_rowid())
        }).await.context("Task join failed")?
    }

    /// List all shift configurations.
    pub async fn list_shifts(&self) -> Result<Vec<ShiftConfig>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, name, start_hour, start_minute, duration_hours, days_mask, created_at FROM shifts ORDER BY id"
            ).context("Prepare list shifts failed")?;
            let rows = stmt.query_map([], |row| {
                Ok(ShiftConfig {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    start_hour: row.get(2)?,
                    start_minute: row.get(3)?,
                    duration_hours: row.get(4)?,
                    days_mask: row.get(5)?,
                    created_at: row.get(6)?,
                })
            }).context("Query list shifts failed")?;
            let mut shifts = Vec::new();
            for row in rows {
                shifts.push(row.context("Row parse shift failed")?);
            }
            Ok(shifts)
        }).await.context("Task join failed")?
    }

    /// Delete a shift configuration.
    pub async fn delete_shift(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute("DELETE FROM shifts WHERE id = ?1", rusqlite::params![id])
                .context("Delete shift failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    /// Query distinct device IDs from telemetry table.
    pub async fn get_telemetry_device_ids(&self) -> Result<Vec<String>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT DISTINCT device_id FROM telemetry ORDER BY device_id")
                .context("Prepare get_telemetry_device_ids failed")?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))
                .context("Query get_telemetry_device_ids failed")?;
            let mut ids = Vec::new();
            for row in rows {
                ids.push(row.context("Row parse get_telemetry_device_ids failed")?);
            }
            Ok(ids)
        }).await.context("Task join failed")?
    }

    /// Query distinct metrics for a device from telemetry table.
    pub async fn get_telemetry_metrics(&self, device_id: &str) -> Result<Vec<String>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT DISTINCT metric FROM telemetry WHERE device_id = ?1 ORDER BY metric"
            ).context("Prepare get_telemetry_metrics failed")?;
            let rows = stmt.query_map(rusqlite::params![device_id], |row| row.get::<_, String>(0))
                .context("Query get_telemetry_metrics failed")?;
            let mut metrics = Vec::new();
            for row in rows {
                metrics.push(row.context("Row parse get_telemetry_metrics failed")?);
            }
            Ok(metrics)
        }).await.context("Task join failed")?
    }

    /// Query telemetry history for a device with optional metric filter.
    pub async fn query_history_multi(
        &self,
        device_id: &str,
        from: i64,
        to: i64,
        metric: Option<&str>,
    ) -> Result<Vec<DataPoint>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        let metric = metric.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut points: Vec<DataPoint> = Vec::new();
            if let Some(ref m) = metric {
                let mut stmt = conn.prepare(
                    "SELECT timestamp, value FROM telemetry
                     WHERE device_id = ?1 AND metric = ?2 AND timestamp >= ?3 AND timestamp <= ?4
                     ORDER BY timestamp LIMIT 5000"
                ).context("Prepare query_history_multi failed")?;
                let rows = stmt.query_map(rusqlite::params![device_id, m, from, to], |row| {
                    Ok(DataPoint { timestamp: row.get(0)?, value: row.get(1)? })
                }).context("Query query_history_multi failed")?;
                for row in rows {
                    points.push(row.context("Row parse query_history_multi failed")?);
                }
            } else {
                let mut stmt = conn.prepare(
                    "SELECT timestamp, value FROM telemetry
                     WHERE device_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3
                     ORDER BY timestamp LIMIT 5000"
                ).context("Prepare query_history_multi failed")?;
                let rows = stmt.query_map(rusqlite::params![device_id, from, to], |row| {
                    Ok(DataPoint { timestamp: row.get(0)?, value: row.get(1)? })
                }).context("Query query_history_multi failed")?;
                for row in rows {
                    points.push(row.context("Row parse query_history_multi failed")?);
                }
            }
            Ok(points)
        }).await.context("Task join failed")?
    }

    // ── Analytics aggregation methods ──

    /// Aggregate raw telemetry data into time buckets.
    pub async fn aggregate_raw(
        &self,
        device_id: &str,
        metric: &str,
        from: i64,
        to: i64,
        interval_secs: i64,
    ) -> Result<Vec<(i64, crate::analytics::AggregatedMetric)>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        let metric = metric.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT
                    (timestamp / ?5) * ?5 as bucket,
                    MIN(value),
                    MAX(value),
                    AVG(value),
                    COUNT(value),
                    MIN(timestamp),
                    MAX(timestamp)
                 FROM telemetry
                 WHERE device_id = ?1 AND metric = ?2
                 AND timestamp >= ?3 AND timestamp <= ?4
                 GROUP BY bucket
                 ORDER BY bucket"
            ).context("Prepare aggregate_raw failed")?;

            let rows = stmt.query_map(
                rusqlite::params![device_id, metric, from, to, interval_secs],
                |row| {
                    let count: i64 = row.get(4)?;
                    let avg: f64 = row.get(3)?;
                    let mut sum_sq_diff = 0.0;
                    // stddev approximation: we need a subquery, but for simplicity
                    // we'll compute it from the result. Here we just return 0.0
                    // and the API layer can compute it if needed.
                    Ok((
                        row.get::<_, i64>(0)?,
                        crate::analytics::AggregatedMetric {
                            metric: metric.clone(),
                            min: row.get(1)?,
                            max: row.get(2)?,
                            avg,
                            stddev: 0.0,
                            count: count as u64,
                            first_ts: row.get(5)?,
                            last_ts: row.get(6)?,
                        },
                    ))
                },
            ).context("Query aggregate_raw failed")?;

            let mut results = Vec::new();
            for row in rows {
                results.push(row.context("Row parse aggregate_raw failed")?);
            }
            Ok(results)
        }).await.context("Task join failed")?
    }

    /// Aggregate hourly telemetry data into time buckets.
    pub async fn aggregate_hourly(
        &self,
        device_id: &str,
        metric: &str,
        from: i64,
        to: i64,
        interval_secs: i64,
    ) -> Result<Vec<(i64, crate::analytics::AggregatedMetric)>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        let metric = metric.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT
                    (bucket_start / ?5) * ?5 as bucket,
                    MIN(min_value),
                    MAX(max_value),
                    AVG(avg_value),
                    SUM(sample_count),
                    MIN(bucket_start),
                    MAX(bucket_start)
                 FROM telemetry_hourly
                 WHERE device_id = ?1 AND metric = ?2
                 AND bucket_start >= ?3 AND bucket_start <= ?4
                 GROUP BY bucket
                 ORDER BY bucket"
            ).context("Prepare aggregate_hourly failed")?;

            let rows = stmt.query_map(
                rusqlite::params![device_id, metric, from, to, interval_secs],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        crate::analytics::AggregatedMetric {
                            metric: metric.clone(),
                            min: row.get(1)?,
                            max: row.get(2)?,
                            avg: row.get(3)?,
                            stddev: 0.0,
                            count: row.get::<_, i64>(4)? as u64,
                            first_ts: row.get(5)?,
                            last_ts: row.get(6)?,
                        },
                    ))
                },
            ).context("Query aggregate_hourly failed")?;

            let mut results = Vec::new();
            for row in rows {
                results.push(row.context("Row parse aggregate_hourly failed")?);
            }
            Ok(results)
        }).await.context("Task join failed")?
    }

    /// Aggregate a single time period into one AggregatedMetric.
    pub async fn aggregate_single_period(
        &self,
        device_id: &str,
        metric: &str,
        from: i64,
        to: i64,
    ) -> Result<crate::analytics::AggregatedMetric> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        let metric = metric.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT
                    MIN(value),
                    MAX(value),
                    AVG(value),
                    COUNT(value),
                    MIN(timestamp),
                    MAX(timestamp)
                 FROM telemetry
                 WHERE device_id = ?1 AND metric = ?2
                 AND timestamp >= ?3 AND timestamp <= ?4"
            ).context("Prepare aggregate_single_period failed")?;

            let row = stmt.query_row(
                rusqlite::params![device_id, metric, from, to],
                |row| {
                    Ok(crate::analytics::AggregatedMetric {
                        metric: metric.clone(),
                        min: row.get(0)?,
                        max: row.get(1)?,
                        avg: row.get(2)?,
                        stddev: 0.0,
                        count: row.get::<_, i64>(3)? as u64,
                        first_ts: row.get(4)?,
                        last_ts: row.get(5)?,
                    })
                },
            );

            match row {
                Ok(m) => Ok(m),
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    Ok(crate::analytics::AggregatedMetric {
                        metric,
                        min: 0.0, max: 0.0, avg: 0.0, stddev: 0.0, count: 0,
                        first_ts: from, last_ts: to,
                    })
                }
                Err(e) => Err(e).context("Query aggregate_single_period failed"),
            }
        }).await.context("Task join failed")?
    }

    // ── Recipe CRUD methods ──

    pub async fn create_recipe(
        &self,
        name: &str,
        category: Option<&str>,
        device_id: &str,
        values: &str,
        description: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let category = category.map(|s| s.to_string());
        let device_id = device_id.to_string();
        let values = values.to_string();
        let description = description.map(|s| s.to_string());
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO recipes (name, category, device_id, values, version, description, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6, ?7)",
                rusqlite::params![name, category, device_id, values, description, now, now],
            ).context("Insert recipe failed")?;
            Ok(conn.last_insert_rowid())
        }).await.context("Task join failed")?
    }

    pub async fn get_recipe(&self, id: i64) -> Result<Option<crate::models::Recipe>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, name, category, device_id, values, version, description, created_at, updated_at
                 FROM recipes WHERE id = ?1"
            ).context("Prepare get_recipe failed")?;
            let mut rows = stmt.query_map(rusqlite::params![id], |row| {
                Ok(crate::models::Recipe {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    device_id: row.get(3)?,
                    values: row.get(4)?,
                    version: row.get(5)?,
                    description: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            }).context("Query get_recipe failed")?;
            match rows.next() {
                Some(row) => Ok(Some(row.context("Row parse get_recipe failed")?)),
                None => Ok(None),
            }
        }).await.context("Task join failed")?
    }

    pub async fn list_recipes(&self, device_id: Option<&str>) -> Result<Vec<crate::models::Recipe>> {
        let conn = self.conn.clone();
        let device_id_owned = device_id.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let recipes = if let Some(ref did) = device_id_owned {
                let mut stmt = conn.prepare(
                    "SELECT id, name, category, device_id, values, version, description, created_at, updated_at
                     FROM recipes WHERE device_id = ?1 ORDER BY updated_at DESC"
                ).context("Prepare list_recipes failed")?;
                let rows = stmt.query_map(rusqlite::params![did], |row| {
                    Ok(crate::models::Recipe {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        category: row.get(2)?,
                        device_id: row.get(3)?,
                        values: row.get(4)?,
                        version: row.get(5)?,
                        description: row.get(6)?,
                        created_at: row.get(7)?,
                        updated_at: row.get(8)?,
                    })
                }).context("Query list_recipes failed")?;
                rows.collect::<Result<Vec<_>, _>>().context("Row parse list_recipes failed")?
            } else {
                let mut stmt = conn.prepare(
                    "SELECT id, name, category, device_id, values, version, description, created_at, updated_at
                     FROM recipes ORDER BY updated_at DESC"
                ).context("Prepare list_recipes failed")?;
                let rows = stmt.query_map([], |row| {
                    Ok(crate::models::Recipe {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        category: row.get(2)?,
                        device_id: row.get(3)?,
                        values: row.get(4)?,
                        version: row.get(5)?,
                        description: row.get(6)?,
                        created_at: row.get(7)?,
                        updated_at: row.get(8)?,
                    })
                }).context("Query list_recipes failed")?;
                rows.collect::<Result<Vec<_>, _>>().context("Row parse list_recipes failed")?
            };
            Ok(recipes)
        }).await.context("Task join failed")?
    }

    pub async fn update_recipe(
        &self,
        id: i64,
        name: Option<&str>,
        category: Option<Option<&str>>,
        values: Option<&str>,
        description: Option<Option<&str>>,
    ) -> Result<bool> {
        let conn = self.conn.clone();
        let name = name.map(|s| s.to_string());
        let category = category.map(|c| c.map(|s| s.to_string()));
        let values = values.map(|s| s.to_string());
        let description = description.map(|d| d.map(|s| s.to_string()));
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let existing: Option<String> = conn
                .query_row(
                    "SELECT values FROM recipes WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .ok();
            match existing {
                None => Ok(false),
                Some(_existing_values) => {
                    let values_to_save = values.unwrap_or(_existing_values);
                    let mut stmt = conn.prepare(
                        "UPDATE recipes SET name = COALESCE(?1, name),
                         category = COALESCE(?2, category),
                         values = ?3,
                         description = COALESCE(?4, description),
                         version = version + 1,
                         updated_at = ?5
                         WHERE id = ?6"
                    ).context("Prepare update_recipe failed")?;
                    let affected = stmt.execute(
                        rusqlite::params![name, category, values_to_save, description, now, id]
                    ).context("Execute update_recipe failed")?;
                    Ok(affected > 0)
                }
            }
        }).await.context("Task join failed")?
    }

    pub async fn delete_recipe(&self, id: i64) -> Result<bool> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let affected = conn.execute(
                "DELETE FROM recipes WHERE id = ?1",
                rusqlite::params![id],
            ).context("Delete recipe failed")?;
            Ok(affected > 0)
        }).await.context("Task join failed")?
    }

    /// Save the current device metric values from cache as a recipe.
    pub async fn save_current_as_recipe(
        &self,
        device_id: &str,
        name: &str,
        category: Option<&str>,
        description: Option<&str>,
        cache: &crate::cache::DeviceCache,
    ) -> Result<i64> {
        let snapshots = cache.get_all().await;
        let metrics = snapshots
            .iter()
            .find(|s| s.device_id == device_id)
            .map(|s| s.metrics.clone())
            .unwrap_or_default();

        let values_json = serde_json::to_string(&metrics)
            .context("Failed to serialize current metric values")?;

        self.create_recipe(name, category, device_id, &values_json, description).await
    }

    // ── Camera CRUD ──

    pub async fn create_camera(
        &self,
        name: &str,
        url: &str,
        enabled: bool,
        snapshot_on_alarm: bool,
        retention_hours: u32,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let url = url.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO cameras (name, url, enabled, snapshot_on_alarm, retention_hours, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![name, url, enabled as i64, snapshot_on_alarm as i64, retention_hours, now],
            )
            .context("Insert camera failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_cameras(&self) -> Result<Vec<Camera>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, url, enabled, snapshot_on_alarm, retention_hours, created_at FROM cameras ORDER BY created_at",
                )
                .context("Prepare list_cameras failed")?;

            let rows = stmt.query_map([], |row| {
                Ok(Camera {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    enabled: row.get::<_, i64>(3)? != 0,
                    snapshot_on_alarm: row.get::<_, i64>(4)? != 0,
                    retention_hours: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .context("Query list_cameras failed")?;

            let mut cameras = Vec::new();
            for row in rows {
                cameras.push(row.context("Row list_cameras failed")?);
            }
            Ok(cameras)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_camera(&self, id: i64) -> Result<Option<Camera>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, url, enabled, snapshot_on_alarm, retention_hours, created_at FROM cameras WHERE id = ?1",
                )
                .context("Prepare get_camera failed")?;

            let mut rows = stmt
                .query_map(rusqlite::params![id], |row| {
                    Ok(Camera {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        url: row.get(2)?,
                        enabled: row.get::<_, i64>(3)? != 0,
                        snapshot_on_alarm: row.get::<_, i64>(4)? != 0,
                        retention_hours: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                })
                .context("Query get_camera failed")?;

            match rows.next() {
                Some(row) => Ok(Some(row.context("Row get_camera failed")?)),
                None => Ok(None),
            }
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_camera(
        &self,
        id: i64,
        name: Option<&str>,
        url: Option<&str>,
        enabled: Option<bool>,
        snapshot_on_alarm: Option<bool>,
        retention_hours: Option<u32>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let name = name.map(|s| s.to_string());
        let url = url.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            if let Some(name) = &name {
                conn.execute("UPDATE cameras SET name = ?1 WHERE id = ?2", rusqlite::params![name, id])
                    .context("Update camera name failed")?;
            }
            if let Some(url) = &url {
                conn.execute("UPDATE cameras SET url = ?1 WHERE id = ?2", rusqlite::params![url, id])
                    .context("Update camera url failed")?;
            }
            if let Some(enabled) = enabled {
                conn.execute("UPDATE cameras SET enabled = ?1 WHERE id = ?2", rusqlite::params![enabled as i64, id])
                    .context("Update camera enabled failed")?;
            }
            if let Some(snapshot_on_alarm) = snapshot_on_alarm {
                conn.execute("UPDATE cameras SET snapshot_on_alarm = ?1 WHERE id = ?2", rusqlite::params![snapshot_on_alarm as i64, id])
                    .context("Update camera snapshot_on_alarm failed")?;
            }
            if let Some(retention_hours) = retention_hours {
                conn.execute("UPDATE cameras SET retention_hours = ?1 WHERE id = ?2", rusqlite::params![retention_hours, id])
                    .context("Update camera retention_hours failed")?;
            }
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_camera(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            // Get file paths for cleanup
            let mut stmt = conn
                .prepare("SELECT file_path FROM camera_snapshots WHERE camera_id = ?1")
                .ok();
            if let Some(ref mut s) = stmt {
                let rows = s.query_map(rusqlite::params![id], |row| {
                    row.get::<_, String>(0)
                }).ok();
                if let Some(rows) = rows {
                    for row in rows.flatten() {
                        let _ = std::fs::remove_file(&row);
                    }
                }
            }
            conn.execute("DELETE FROM cameras WHERE id = ?1", rusqlite::params![id])
                .context("Delete camera failed")
                .map(|_| ())
        })
        .await
        .context("Task join failed")?
    }

    // ── Camera Snapshot CRUD ──

    pub async fn create_camera_snapshot(
        &self,
        camera_id: i64,
        timestamp: i64,
        alarm_source: Option<&str>,
        file_path: &str,
        file_size: u64,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let alarm_source = alarm_source.map(|s| s.to_string());
        let file_path = file_path.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO camera_snapshots (camera_id, timestamp, alarm_source, file_path, file_size, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![camera_id, timestamp, alarm_source, file_path, file_size, crate::timeutil::now_secs()],
            )
            .context("Insert camera snapshot failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_camera_snapshots(
        &self,
        camera_id: i64,
        limit: usize,
    ) -> Result<Vec<CameraSnapshot>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, camera_id, timestamp, alarm_source, file_path, file_size
                     FROM camera_snapshots
                     WHERE camera_id = ?1
                     ORDER BY timestamp DESC
                     LIMIT ?2",
                )
                .context("Prepare list_camera_snapshots failed")?;

            let rows = stmt
                .query_map(rusqlite::params![camera_id, limit as i64], |row| {
                    Ok(CameraSnapshot {
                        id: row.get(0)?,
                        camera_id: row.get(1)?,
                        timestamp: row.get(2)?,
                        alarm_source: row.get(3)?,
                        file_path: row.get(4)?,
                        file_size: row.get(5)?,
                    })
                })
                .context("Query list_camera_snapshots failed")?;

            let mut snapshots = Vec::new();
            for row in rows {
                snapshots.push(row.context("Row list_camera_snapshots failed")?);
            }
            Ok(snapshots)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_camera_snapshot(&self, snapshot_id: i64) -> Result<Option<CameraSnapshot>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, camera_id, timestamp, alarm_source, file_path, file_size
                     FROM camera_snapshots WHERE id = ?1",
                )
                .context("Prepare get_camera_snapshot failed")?;

            let mut rows = stmt
                .query_map(rusqlite::params![snapshot_id], |row| {
                    Ok(CameraSnapshot {
                        id: row.get(0)?,
                        camera_id: row.get(1)?,
                        timestamp: row.get(2)?,
                        alarm_source: row.get(3)?,
                        file_path: row.get(4)?,
                        file_size: row.get(5)?,
                    })
                })
                .context("Query get_camera_snapshot failed")?;

            match rows.next() {
                Some(row) => Ok(Some(row.context("Row get_camera_snapshot failed")?)),
                None => Ok(None),
            }
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_camera_snapshot(&self, snapshot_id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            // Get file path for cleanup
            let file_path: Option<String> = conn
                .query_row(
                    "SELECT file_path FROM camera_snapshots WHERE id = ?1",
                    rusqlite::params![snapshot_id],
                    |row| row.get(0),
                )
                .ok();
            if let Some(path) = &file_path {
                let _ = std::fs::remove_file(path);
            }
            conn.execute("DELETE FROM camera_snapshots WHERE id = ?1", rusqlite::params![snapshot_id])
                .context("Delete camera snapshot failed")
                .map(|_| ())
        })
        .await
        .context("Task join failed")?
    }

    /// Delete snapshots older than the camera's retention period.
    pub async fn cleanup_expired_snapshots(&self, camera_id: i64, retention_hours: u32) -> Result<u64> {
        let cutoff = crate::timeutil::now_secs() - (retention_hours as i64 * 3600);
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            // Get file paths for cleanup
            let mut stmt = conn
                .prepare("SELECT file_path FROM camera_snapshots WHERE camera_id = ?1 AND timestamp < ?2")
                .ok();
            if let Some(ref mut s) = stmt {
                let rows = s.query_map(rusqlite::params![camera_id, cutoff], |row| {
                    row.get::<_, String>(0)
                }).ok();
                if let Some(rows) = rows {
                    for row in rows.flatten() {
                        let _ = std::fs::remove_file(&row);
                    }
                }
            }
            conn.execute(
                "DELETE FROM camera_snapshots WHERE camera_id = ?1 AND timestamp < ?2",
                rusqlite::params![camera_id, cutoff],
            )
            .context("Cleanup expired snapshots failed")
            .map(|n| n as u64)
        })
        .await
        .context("Task join failed")?
    }
}

// ── Security audit log (v0.22.0 IEC 62443) ──

impl Storage {
    /// Insert a security event into the append-only security_log table.
    pub async fn insert_security_event(
        &self,
        event: &crate::models::CreateSecurityEvent,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let event_type = event.event_type.clone();
        let severity = event.severity.clone();
        let source_ip = event.source_ip.clone();
        let user_agent = event.user_agent.clone();
        let username = event.username.clone();
        let details = event.details.clone();
        let now = crate::timeutil::now_secs();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO security_log (event_type, severity, source_ip, user_agent, username, details, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![event_type, severity, source_ip, user_agent, username, details, now],
            ).context("Insert security event failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    /// List security events with optional filters and pagination.
    pub async fn list_security_events(
        &self,
        event_type: Option<&str>,
        severity: Option<&str>,
        from: Option<i64>,
        to: Option<i64>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<crate::models::SecurityEvent>> {
        let conn = self.conn.clone();
        let event_type = event_type.map(|s| s.to_string());
        let severity = severity.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let mut sql = String::from(
                "SELECT id, event_type, severity, source_ip, user_agent, username, details, created_at \
                 FROM security_log WHERE 1=1"
            );
            let mut param_idx = 1;
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(et) = event_type {
                sql.push_str(&format!(" AND event_type = ?{}", param_idx));
                param_values.push(Box::new(et.to_string()));
                param_idx += 1;
            }
            if let Some(sev) = severity {
                sql.push_str(&format!(" AND severity = ?{}", param_idx));
                param_values.push(Box::new(sev.to_string()));
                param_idx += 1;
            }
            if let Some(f) = from {
                sql.push_str(&format!(" AND created_at >= ?{}", param_idx));
                param_values.push(Box::new(f));
                param_idx += 1;
            }
            if let Some(t) = to {
                sql.push_str(&format!(" AND created_at <= ?{}", param_idx));
                param_values.push(Box::new(t));
                param_idx += 1;
            }

            sql.push_str(&format!(" ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}", param_idx, param_idx + 1));
            param_values.push(Box::new(limit));
            param_values.push(Box::new(offset));

            let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

            let mut stmt = conn.prepare(&sql).context("Prepare security events query failed")?;
            let events = stmt.query_map(params.as_slice(), |row| {
                Ok(crate::models::SecurityEvent {
                    id: row.get(0)?,
                    event_type: row.get(1)?,
                    severity: row.get(2)?,
                    source_ip: row.get(3)?,
                    user_agent: row.get(4)?,
                    username: row.get(5)?,
                    details: row.get(6)?,
                    created_at: row.get(7)?,
                })
            }).context("Query security events failed")?
            .collect::<Result<Vec<_>, _>>()
            .context("Map security events failed")?;

            Ok(events)
        }).await.context("Task join failed")?
    }

    /// Export security events as CSV string for external audit tools.
    pub async fn export_security_events_csv(
        &self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Result<String> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let mut sql = String::from(
                "SELECT id, event_type, severity, source_ip, user_agent, username, details, created_at \
                 FROM security_log WHERE 1=1"
            );
            let mut param_idx = 1;
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(f) = from {
                sql.push_str(&format!(" AND created_at >= ?{}", param_idx));
                param_values.push(Box::new(f));
                param_idx += 1;
            }
            if let Some(t) = to {
                sql.push_str(&format!(" AND created_at <= ?{}", param_idx));
                param_values.push(Box::new(t));
                param_idx += 1;
            }

            sql.push_str(" ORDER BY created_at ASC");
            let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

            let mut stmt = conn.prepare(&sql).context("Prepare security CSV query failed")?;
            let rows = stmt.query_map(params.as_slice(), |row| {
                Ok(crate::models::SecurityEvent {
                    id: row.get(0)?,
                    event_type: row.get(1)?,
                    severity: row.get(2)?,
                    source_ip: row.get(3)?,
                    user_agent: row.get(4)?,
                    username: row.get(5)?,
                    details: row.get(6)?,
                    created_at: row.get(7)?,
                })
            }).context("Query security CSV failed")?;

            let mut csv = String::from("id,event_type,severity,source_ip,user_agent,username,details,created_at\n");
            for row in rows {
                let event = row.context("Map security CSV row failed")?;
                csv.push_str(&format!(
                    "{},\"{}\",\"{}\",{},{},{},{},{}\n",
                    event.id,
                    event.event_type.replace('"', "\"\""),
                    event.severity.replace('"', "\"\""),
                    event.source_ip.as_deref().unwrap_or(""),
                    event.user_agent.as_deref().unwrap_or("").replace('"', "\"\""),
                    event.username.as_deref().unwrap_or("").replace('"', "\"\""),
                    event.details.as_deref().unwrap_or("").replace('"', "\"\"").replace('\n', "\\n"),
                    event.created_at,
                ));
            }

            Ok(csv)
        }).await.context("Task join failed")?
    }

    /// Get the current security thresholds configuration.
    pub async fn get_security_thresholds(&self) -> Result<crate::models::SecurityThresholds> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT failed_login_threshold, failed_login_window_secs, \
                        suspicious_401_threshold, suspicious_401_window_secs, \
                        rate_limit_threshold, rate_limit_window_secs \
                 FROM security_thresholds WHERE id = 1"
            ).context("Prepare thresholds query failed")?;

            let thresholds = stmt.query_row([], |row| {
                Ok(crate::models::SecurityThresholds {
                    failed_login_threshold: row.get(0)?,
                    failed_login_window_secs: row.get(1)?,
                    suspicious_401_threshold: row.get(2)?,
                    suspicious_401_window_secs: row.get(3)?,
                    rate_limit_threshold: row.get(4)?,
                    rate_limit_window_secs: row.get(5)?,
                })
            }).context("Query thresholds failed")?;

            Ok(thresholds)
        }).await.context("Task join failed")?
    }

    /// Update security thresholds configuration.
    pub async fn update_security_thresholds(
        &self,
        thresholds: &crate::models::SecurityThresholds,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let t = thresholds.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE security_thresholds SET \
                 failed_login_threshold = ?1, failed_login_window_secs = ?2, \
                 suspicious_401_threshold = ?3, suspicious_401_window_secs = ?4, \
                 rate_limit_threshold = ?5, rate_limit_window_secs = ?6 \
                 WHERE id = 1",
                rusqlite::params![
                    t.failed_login_threshold, t.failed_login_window_secs,
                    t.suspicious_401_threshold, t.suspicious_401_window_secs,
                    t.rate_limit_threshold, t.rate_limit_window_secs,
                ],
            ).context("Update thresholds failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    // ── Annotation CRUD (v0.23.0) ──

    pub async fn create_annotation(&self, req: &CreateAnnotationRequest, author_id: i64) -> Result<i64> {
        let conn = self.conn.clone();
        let req = req.clone();
        let author_id = author_id;
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO annotations (tag_id, annotation_text, data_timestamp, author_id, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![req.tag_id, req.annotation_text, req.data_timestamp, author_id, now, now],
            ).context("Insert annotation failed")?;
            Ok(conn.last_insert_rowid())
        }).await.context("Task join failed")?
    }

    pub async fn list_annotations(
        &self,
        tag_id: Option<&str>,
        from_ts: Option<i64>,
        to_ts: Option<i64>,
    ) -> Result<Vec<Annotation>> {
        let conn = self.conn.clone();
        let tag_id = tag_id.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut sql = String::from(
                "SELECT id, tag_id, annotation_text, data_timestamp, author_id, created_at, updated_at
                 FROM annotations WHERE 1=1"
            );
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            if let Some(ref t) = tag_id {
                sql.push_str(" AND tag_id = ?");
                params.push(Box::new(t.clone()));
            }
            if let Some(from) = from_ts {
                sql.push_str(" AND data_timestamp >= ?");
                params.push(Box::new(from));
            }
            if let Some(to) = to_ts {
                sql.push_str(" AND data_timestamp <= ?");
                params.push(Box::new(to));
            }
            sql.push_str(" ORDER BY data_timestamp DESC");
            let mut stmt = conn.prepare(&sql).context("Prepare annotations failed")?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(param_refs.as_slice(), |row| {
                Ok(Annotation {
                    id: row.get(0)?,
                    tag_id: row.get(1)?,
                    annotation_text: row.get(2)?,
                    data_timestamp: row.get(3)?,
                    author_id: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            }).context("Query annotations failed")?;
            rows.collect::<Result<Vec<_>, _>>().context("Collect annotations failed")
        }).await.context("Task join failed")?
    }

    pub async fn get_annotation(&self, id: i64) -> Result<Option<Annotation>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let result = conn.query_row(
                "SELECT id, tag_id, annotation_text, data_timestamp, author_id, created_at, updated_at
                 FROM annotations WHERE id = ?1",
                rusqlite::params![id],
                |row| {
                    Ok(Annotation {
                        id: row.get(0)?,
                        tag_id: row.get(1)?,
                        annotation_text: row.get(2)?,
                        data_timestamp: row.get(3)?,
                        author_id: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                },
            ).ok();
            Ok(result)
        }).await.context("Task join failed")?
    }

    pub async fn update_annotation(&self, id: i64, req: &UpdateAnnotationRequest) -> Result<()> {
        let conn = self.conn.clone();
        let req = req.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            if let Some(ref text) = req.annotation_text {
                conn.execute(
                    "UPDATE annotations SET annotation_text = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![text, now, id],
                ).context("Update annotation failed")?;
            }
            Ok(())
        }).await.context("Task join failed")?
    }

    pub async fn delete_annotation(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "DELETE FROM annotations WHERE id = ?1",
                rusqlite::params![id],
            ).context("Delete annotation failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    // ── Shift Handover CRUD (v0.23.0) ──

    pub async fn create_shift_handover(&self, req: &CreateShiftHandoverRequest, author_id: i64) -> Result<i64> {
        let conn = self.conn.clone();
        let req = req.clone();
        let author_id = author_id;
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO shift_handovers (author_id, shift_date, shift_period, what_happened, whats_pending, what_to_watch, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    author_id,
                    req.shift_date,
                    req.shift_period,
                    req.what_happened.unwrap_or_default(),
                    req.whats_pending.unwrap_or_default(),
                    req.what_to_watch.unwrap_or_default(),
                    now,
                ],
            ).context("Insert shift handover failed")?;
            Ok(conn.last_insert_rowid())
        }).await.context("Task join failed")?
    }

    pub async fn list_shift_handovers(
        &self,
        shift_date: Option<&str>,
        shift_period: Option<&str>,
    ) -> Result<Vec<ShiftHandover>> {
        let conn = self.conn.clone();
        let shift_date = shift_date.map(|s| s.to_string());
        let shift_period = shift_period.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut sql = String::from(
                "SELECT id, author_id, shift_date, shift_period, what_happened, whats_pending, what_to_watch, created_at
                 FROM shift_handovers WHERE 1=1"
            );
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            if let Some(ref d) = shift_date {
                sql.push_str(" AND shift_date = ?");
                params.push(Box::new(d.clone()));
            }
            if let Some(ref p) = shift_period {
                sql.push_str(" AND shift_period = ?");
                params.push(Box::new(p.clone()));
            }
            sql.push_str(" ORDER BY shift_date DESC, shift_period ASC");
            let mut stmt = conn.prepare(&sql).context("Prepare shift handovers failed")?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(param_refs.as_slice(), |row| {
                Ok(ShiftHandover {
                    id: row.get(0)?,
                    author_id: row.get(1)?,
                    shift_date: row.get(2)?,
                    shift_period: row.get(3)?,
                    what_happened: row.get(4)?,
                    whats_pending: row.get(5)?,
                    what_to_watch: row.get(6)?,
                    created_at: row.get(7)?,
                })
            }).context("Query shift handovers failed")?;
            rows.collect::<Result<Vec<_>, _>>().context("Collect shift handovers failed")
        }).await.context("Task join failed")?
    }

    pub async fn get_shift_handover(&self, id: i64) -> Result<Option<ShiftHandover>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let result = conn.query_row(
                "SELECT id, author_id, shift_date, shift_period, what_happened, whats_pending, what_to_watch, created_at
                 FROM shift_handovers WHERE id = ?1",
                rusqlite::params![id],
                |row| {
                    Ok(ShiftHandover {
                        id: row.get(0)?,
                        author_id: row.get(1)?,
                        shift_date: row.get(2)?,
                        shift_period: row.get(3)?,
                        what_happened: row.get(4)?,
                        whats_pending: row.get(5)?,
                        what_to_watch: row.get(6)?,
                        created_at: row.get(7)?,
                    })
                },
            ).ok();
            Ok(result)
        }).await.context("Task join failed")?
    }

    // ── User Notification CRUD (v0.23.0) ──

    pub async fn create_notification(&self, req: &CreateNotificationRequest) -> Result<i64> {
        let conn = self.conn.clone();
        let req = req.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            conn.execute(
                "INSERT INTO user_notifications (user_id, from_user_id, notification_type, message, reference_id, read, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
                rusqlite::params![
                    req.user_id,
                    req.from_user_id,
                    req.notification_type,
                    req.message,
                    req.reference_id,
                    now,
                ],
            ).context("Insert notification failed")?;
            Ok(conn.last_insert_rowid())
        }).await.context("Task join failed")?
    }

    pub async fn list_user_notifications(
        &self,
        user_id: i64,
        unread_only: bool,
    ) -> Result<Vec<UserNotification>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = if unread_only {
                (
                    "SELECT id, user_id, from_user_id, notification_type, message, reference_id, read, created_at
                     FROM user_notifications WHERE user_id = ?1 AND read = 0
                     ORDER BY created_at DESC",
                    vec![Box::new(user_id)],
                )
            } else {
                (
                    "SELECT id, user_id, from_user_id, notification_type, message, reference_id, read, created_at
                     FROM user_notifications WHERE user_id = ?1
                     ORDER BY created_at DESC",
                    vec![Box::new(user_id)],
                )
            };
            let mut stmt = conn.prepare(sql).context("Prepare notifications failed")?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(param_refs.as_slice(), |row| {
                Ok(UserNotification {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    from_user_id: row.get(2)?,
                    notification_type: row.get(3)?,
                    message: row.get(4)?,
                    reference_id: row.get(5)?,
                    read: row.get::<_, i64>(6)? != 0,
                    created_at: row.get(7)?,
                })
            }).context("Query notifications failed")?;
            rows.collect::<Result<Vec<_>, _>>().context("Collect notifications failed")
        }).await.context("Task join failed")?
    }

    pub async fn mark_notification_read(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE user_notifications SET read = 1 WHERE id = ?1",
                rusqlite::params![id],
            ).context("Mark notification read failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    pub async fn dismiss_notification(&self, id: i64) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "DELETE FROM user_notifications WHERE id = ?1",
                rusqlite::params![id],
            ).context("Dismiss notification failed")?;
            Ok(())
        }).await.context("Task join failed")?
    }

    pub async fn cleanup_old_notifications(&self) -> Result<u64> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let cutoff = crate::timeutil::now_secs() - 30 * 24 * 3600;
            let affected = conn.execute(
                "DELETE FROM user_notifications WHERE created_at < ?1",
                rusqlite::params![cutoff],
            ).context("Cleanup old notifications failed")?;
            Ok(affected as u64)
        }).await.context("Task join failed")?
    }
}

// ── Command & Control methods (v0.24.0) ──

impl Storage {
    pub async fn insert_command(&self, cmd: &Command) -> Result<()> {
        let conn = self.conn.clone();
        let cmd = cmd.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO commands (id, data_source_id, device_id, protocol, value, status, policy,
                     scheduled_at, retry_count, max_retries, error_message, created_at, sent_at, acked_at,
                     resolved_at, created_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                rusqlite::params![
                    cmd.id, cmd.data_source_id, cmd.device_id, cmd.protocol, cmd.value,
                    cmd.status, cmd.policy, cmd.scheduled_at, cmd.retry_count, cmd.max_retries,
                    cmd.error_message, cmd.created_at, cmd.sent_at, cmd.acked_at, cmd.resolved_at,
                    cmd.created_by,
                ],
            )
            .context("Insert command failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_command_status(
        &self,
        command_id: &str,
        new_status: &str,
        error_message: Option<&str>,
        changed_by: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let command_id = command_id.to_string();
        let new_status = new_status.to_string();
        let error_message = error_message.map(|s| s.to_string());
        let changed_by = changed_by.to_string();
        let reason = reason.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            let now = crate::timeutil::now_secs();

            // Fetch current status for audit log
            let old_status: String = tx
                .query_row(
                    "SELECT status FROM commands WHERE id = ?",
                    rusqlite::params![command_id],
                    |row| row.get(0),
                )
                .context("Command not found")?;

            // Build SQL and params based on new status (using unnumbered ? placeholders
            // bound positionally by rusqlite to avoid off-by-one errors).
            match new_status.as_str() {
                "sent" => {
                    tx.execute(
                        "UPDATE commands SET status = ?, sent_at = ?, error_message = ? WHERE id = ?",
                        rusqlite::params![new_status, now, error_message, command_id],
                    )
                    .context("Update command status failed")?;
                }
                "acked" => {
                    tx.execute(
                        "UPDATE commands SET status = ?, acked_at = ?, resolved_at = ?, error_message = ? WHERE id = ?",
                        rusqlite::params![new_status, now, now, error_message, command_id],
                    )
                    .context("Update command status failed")?;
                }
                "failed" | "expired" | "cancelled" => {
                    tx.execute(
                        "UPDATE commands SET status = ?, resolved_at = ?, error_message = ? WHERE id = ?",
                        rusqlite::params![new_status, now, error_message, command_id],
                    )
                    .context("Update command status failed")?;
                }
                "queued" => {
                    tx.execute(
                        "UPDATE commands SET status = ?, retry_count = retry_count + 1, error_message = ? WHERE id = ?",
                        rusqlite::params![new_status, error_message, command_id],
                    )
                    .context("Update command status failed")?;
                }
                _ => {
                    tx.execute(
                        "UPDATE commands SET status = ?, error_message = ? WHERE id = ?",
                        rusqlite::params![new_status, error_message, command_id],
                    )
                    .context("Update command status failed")?;
                }
            }

            // Insert audit log entry
            tx.execute(
                "INSERT INTO command_audit_log (command_id, old_status, new_status, changed_by, reason, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
                rusqlite::params![command_id, old_status, new_status, changed_by, reason, now],
            )
            .context("Insert command audit log failed")?;

            tx.commit().context("Commit failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_command(&self, command_id: &str) -> Result<Option<Command>> {
        let conn = self.conn.clone();
        let command_id = command_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let result = conn.query_row(
                "SELECT id, data_source_id, device_id, protocol, value, status, policy,
                        scheduled_at, retry_count, max_retries, error_message, created_at,
                        sent_at, acked_at, resolved_at, created_by
                 FROM commands WHERE id = ?1",
                rusqlite::params![command_id],
                |row| {
                    Ok(Command {
                        id: row.get(0)?,
                        data_source_id: row.get(1)?,
                        device_id: row.get(2)?,
                        protocol: row.get(3)?,
                        value: row.get(4)?,
                        status: row.get(5)?,
                        policy: row.get(6)?,
                        scheduled_at: row.get(7)?,
                        retry_count: row.get(8)?,
                        max_retries: row.get(9)?,
                        error_message: row.get(10)?,
                        created_at: row.get(11)?,
                        sent_at: row.get(12)?,
                        acked_at: row.get(13)?,
                        resolved_at: row.get(14)?,
                        created_by: row.get(15)?,
                    })
                },
            );
            match result {
                Ok(cmd) => Ok(Some(cmd)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e).context("Get command failed"),
            }
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_commands(&self, query: &CommandQuery) -> Result<Vec<Command>> {
        let conn = self.conn.clone();
        let tag = query.tag.clone();
        let status = query.status.clone();
        let device = query.device.clone();
        let from = query.from;
        let to = query.to;
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let mut sql = String::from(
                "SELECT id, data_source_id, device_id, protocol, value, status, policy,
                        scheduled_at, retry_count, max_retries, error_message, created_at,
                        sent_at, acked_at, resolved_at, created_by
                 FROM commands WHERE 1=1",
            );
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(ref s) = status {
                sql.push_str(" AND status = ?");
                params.push(Box::new(s.clone()));
            }
            if let Some(ref d) = device {
                // device_id is INTEGER in commands; the query param is a device identifier string
                if let Ok(id) = d.parse::<i64>() {
                    sql.push_str(" AND device_id = ?");
                    params.push(Box::new(id));
                } else {
                    // Non-numeric device id; no match possible
                    sql.push_str(" AND 0=1");
                }
            }
            if let Some(f) = from {
                sql.push_str(" AND created_at >= ?");
                params.push(Box::new(f));
            }
            if let Some(t) = to {
                sql.push_str(" AND created_at <= ?");
                params.push(Box::new(t));
            }
            if let Some(ref t) = tag {
                // tag matches data_source_id via key lookup
                sql.push_str(" AND data_source_id IN (SELECT id FROM data_sources WHERE key = ?)");
                params.push(Box::new(t.clone()));
            }

            sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
            params.push(Box::new(limit));
            params.push(Box::new(offset));

            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let mut stmt = conn.prepare(&sql).context("Prepare list_commands failed")?;
            let rows = stmt
                .query_map(param_refs.as_slice(), |row| {
                    Ok(Command {
                        id: row.get(0)?,
                        data_source_id: row.get(1)?,
                        device_id: row.get(2)?,
                        protocol: row.get(3)?,
                        value: row.get(4)?,
                        status: row.get(5)?,
                        policy: row.get(6)?,
                        scheduled_at: row.get(7)?,
                        retry_count: row.get(8)?,
                        max_retries: row.get(9)?,
                        error_message: row.get(10)?,
                        created_at: row.get(11)?,
                        sent_at: row.get(12)?,
                        acked_at: row.get(13)?,
                        resolved_at: row.get(14)?,
                        created_by: row.get(15)?,
                    })
                })
                .context("Query list_commands failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_queued_commands(&self) -> Result<Vec<Command>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, data_source_id, device_id, protocol, value, status, policy,
                        scheduled_at, retry_count, max_retries, error_message, created_at,
                        sent_at, acked_at, resolved_at, created_by
                 FROM commands
                 WHERE status IN ('queued', 'pending')
                 ORDER BY created_at ASC",
            ).context("Prepare get_queued_commands failed")?;
            let rows = stmt.query_map([], |row| {
                Ok(Command {
                    id: row.get(0)?,
                    data_source_id: row.get(1)?,
                    device_id: row.get(2)?,
                    protocol: row.get(3)?,
                    value: row.get(4)?,
                    status: row.get(5)?,
                    policy: row.get(6)?,
                    scheduled_at: row.get(7)?,
                    retry_count: row.get(8)?,
                    max_retries: row.get(9)?,
                    error_message: row.get(10)?,
                    created_at: row.get(11)?,
                    sent_at: row.get(12)?,
                    acked_at: row.get(13)?,
                    resolved_at: row.get(14)?,
                    created_by: row.get(15)?,
                })
            }).context("Query get_queued_commands failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_command_audit(&self, command_id: &str) -> Result<Vec<CommandAuditEntry>> {
        let conn = self.conn.clone();
        let command_id = command_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, command_id, old_status, new_status, changed_by, reason, created_at
                 FROM command_audit_log
                 WHERE command_id = ?1
                 ORDER BY created_at ASC",
            ).context("Prepare get_command_audit failed")?;
            let rows = stmt.query_map(rusqlite::params![command_id], |row| {
                Ok(CommandAuditEntry {
                    id: row.get(0)?,
                    command_id: row.get(1)?,
                    old_status: row.get(2)?,
                    new_status: row.get(3)?,
                    changed_by: row.get(4)?,
                    reason: row.get(5)?,
                    created_at: row.get(6)?,
                })
            }).context("Query get_command_audit failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_write_policy(&self, data_source_id: i64) -> Result<Option<WritePolicy>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let result = conn.query_row(
                "SELECT writable, write_min, write_max, write_max_rate, write_requires_reason, write_deadband
                 FROM data_sources WHERE id = ?1",
                rusqlite::params![data_source_id],
                |row| {
                    let writable: i64 = row.get(0)?;
                    let write_requires_reason: i64 = row.get(4)?;
                    Ok(WritePolicy {
                        writable: writable != 0,
                        write_min: row.get(1)?,
                        write_max: row.get(2)?,
                        write_max_rate: row.get(3)?,
                        write_requires_reason: write_requires_reason != 0,
                        write_deadband: row.get(5)?,
                    })
                },
            );
            match result {
                Ok(policy) => Ok(Some(policy)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e).context("Get write policy failed"),
            }
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_write_policy(&self, data_source_id: i64, update: &WritePolicyUpdate) -> Result<()> {
        let conn = self.conn.clone();
        let writable = update.writable;
        let write_min = update.write_min.clone();
        let write_max = update.write_max.clone();
        let write_max_rate = update.write_max_rate.clone();
        let write_requires_reason = update.write_requires_reason;
        let write_deadband = update.write_deadband.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let mut sets: Vec<String> = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(v) = writable {
                sets.push(format!("writable = ?{}", sets.len() + 1));
                params.push(Box::new(v as i64));
            }
            if let Some(ref v) = write_min {
                sets.push(format!("write_min = ?{}", sets.len() + 1));
                params.push(Box::new(*v));
            }
            if let Some(ref v) = write_max {
                sets.push(format!("write_max = ?{}", sets.len() + 1));
                params.push(Box::new(*v));
            }
            if let Some(ref v) = write_max_rate {
                sets.push(format!("write_max_rate = ?{}", sets.len() + 1));
                params.push(Box::new(*v));
            }
            if let Some(v) = write_requires_reason {
                sets.push(format!("write_requires_reason = ?{}", sets.len() + 1));
                params.push(Box::new(v as i64));
            }
            if let Some(ref v) = write_deadband {
                sets.push(format!("write_deadband = ?{}", sets.len() + 1));
                params.push(Box::new(*v));
            }

            if sets.is_empty() {
                return Ok(());
            }

            let sql = format!(
                "UPDATE data_sources SET {} WHERE id = ?{}",
                sets.join(", "),
                sets.len() + 1,
            );
            params.push(Box::new(data_source_id));

            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            conn.execute(&sql, param_refs.as_slice())
                .context("Update write policy failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn count_recent_writes(&self, data_source_id: i64, window_secs: i64) -> Result<i64> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            let cutoff = now - window_secs;
            // NOTE: We count by created_at (when the command was queued) rather than
            // sent_at (when it was actually sent). This means commands still sitting in
            // "pending"/"queued" also count against the rate limit. The trade-off is that
            // a burst of queued commands won't slip past the limit — the user gets rate-
            // limited at submission time. If we used sent_at only, a flood of pending
            // commands could all be dispatched once the processor catches up.
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM commands WHERE data_source_id = ? AND created_at >= ?",
                    rusqlite::params![data_source_id, cutoff],
                    |row| row.get(0),
                )
                .context("Count recent writes failed")?;
            Ok(count)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_data_source_device_id(&self, data_source_id: i64) -> Result<Option<(i64, String)>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            // data_sources has device_id TEXT and source_type TEXT.
            // For modbus, device_id is a string id. For opcua/s7/enip, it's a stringified integer.
            // We parse device_id as i64 and return source_type as protocol.
            let result = conn.query_row(
                "SELECT device_id, source_type FROM data_sources WHERE id = ?1",
                rusqlite::params![data_source_id],
                |row| {
                    let device_id_str: String = row.get(0)?;
                    let protocol: String = row.get(1)?;
                    Ok((device_id_str, protocol))
                },
            );
            match result {
                Ok((device_id_str, protocol)) => {
                    match device_id_str.parse::<i64>() {
                        Ok(id) => Ok(Some((id, protocol))),
                        Err(_) => {
                            // device_id is non-numeric (e.g., modbus text id).
                            // Try to look up integer id from modbus_devices table.
                            let modbus_id: i64 = conn
                                .query_row(
                                    "SELECT COUNT(*) FROM modbus_devices WHERE id = ?1",
                                    rusqlite::params![device_id_str],
                                    |row| row.get::<_, i64>(0),
                                )
                                .unwrap_or(0);
                            if modbus_id > 0 {
                                // Modbus device exists but has text id; return 0 as device_id marker
                                // The caller will need to handle modbus text IDs separately
                                Ok(Some((0i64, protocol)))
                            } else {
                                Ok(None)
                            }
                        }
                    }
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e).context("Get data source device id failed"),
            }
        })
        .await
        .context("Task join failed")?
    }

    /// Find a data source by device_id and key. Returns the data source ID if found.
    pub async fn find_data_source_by_device_and_key(&self, device_id: &str, key: &str) -> Result<Option<DataSource>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        let key = key.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let result = conn.query_row(
                "SELECT id, key, name, unit, color, min_val, max_val, warn_val, crit_val, base_val, sort_order, decimals, source_type, source_config, device_id, equipment_id, chart_type FROM data_sources WHERE device_id = ?1 AND key = ?2",
                rusqlite::params![device_id, key],
                |row| {
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
                },
            );
            match result {
                Ok(ds) => Ok(Some(ds)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e).context("Find data source by device and key failed"),
            }
        })
        .await
        .context("Task join failed")?
    }

    /// Get the key of a data source by its ID. Returns None if not found.
    pub async fn get_data_source_key(&self, data_source_id: i64) -> Result<Option<String>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let result = conn.query_row(
                "SELECT key FROM data_sources WHERE id = ?1",
                rusqlite::params![data_source_id],
                |row| row.get::<_, String>(0),
            );
            match result {
                Ok(key) => Ok(Some(key)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e).context("Get data source key failed"),
            }
        })
        .await
        .context("Task join failed")?
    }

    // ── Sim Profile CRUD methods ──

    pub async fn list_sim_profiles(&self) -> anyhow::Result<Vec<crate::models::SimProfile>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, name, data_type, config, created_at FROM sim_profiles ORDER BY name")
                .context("Prepare list_sim_profiles failed")?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(crate::models::SimProfile {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        data_type: row.get(2)?,
                        config: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                })
                .context("Query list_sim_profiles failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row parse list_sim_profiles failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_sim_profile(&self, id: i64) -> anyhow::Result<Option<crate::models::SimProfile>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, name, data_type, config, created_at FROM sim_profiles WHERE id = ?1")
                .context("Prepare get_sim_profile failed")?;
            let mut rows = stmt
                .query_map(rusqlite::params![id], |row| {
                    Ok(crate::models::SimProfile {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        data_type: row.get(2)?,
                        config: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                })
                .context("Query get_sim_profile failed")?;
            match rows.next() {
                Some(row) => Ok(Some(row.context("Row parse get_sim_profile failed")?)),
                None => Ok(None),
            }
        })
        .await
        .context("Task join failed")?
    }

    pub async fn create_sim_profile(
        &self,
        name: &str,
        data_type: &str,
        config: &str,
    ) -> anyhow::Result<i64> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let data_type = data_type.to_string();
        let config = config.to_string();
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO sim_profiles (name, data_type, config, created_at) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![name, data_type, config, now],
            )
            .context("Insert sim_profile failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_sim_profile(
        &self,
        id: i64,
        name: Option<&str>,
        data_type: Option<&str>,
        config: Option<&str>,
    ) -> anyhow::Result<bool> {
        let conn = self.conn.clone();
        let name = name.map(|s| s.to_string());
        let data_type = data_type.map(|s| s.to_string());
        let config = config.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let existing = conn
                .query_row(
                    "SELECT name, data_type, config FROM sim_profiles WHERE id = ?1",
                    rusqlite::params![id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
                )
                .ok();
            match existing {
                None => Ok(false),
                Some((cur_name, cur_data_type, cur_config)) => {
                    let name = name.unwrap_or(cur_name);
                    let data_type = data_type.unwrap_or(cur_data_type);
                    let config = config.unwrap_or(cur_config);
                    let affected = conn
                        .execute(
                            "UPDATE sim_profiles SET name = ?1, data_type = ?2, config = ?3 WHERE id = ?4",
                            rusqlite::params![name, data_type, config, id],
                        )
                        .context("Update sim_profile failed")?;
                    Ok(affected > 0)
                }
            }
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_sim_profile(&self, id: i64) -> anyhow::Result<bool> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let affected = conn
                .execute("DELETE FROM sim_profiles WHERE id = ?1", rusqlite::params![id])
                .context("Delete sim_profile failed")?;
            Ok(affected > 0)
        })
        .await
        .context("Task join failed")?
    }

    // ── Device simulation get/set ──

    pub async fn set_device_simulation(
        &self,
        device_id: &str,
        sim_mode: bool,
        sim_profile_id: Option<i64>,
        sim_config: Option<&str>,
    ) -> anyhow::Result<bool> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        let sim_config = sim_config.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let affected = conn
                .execute(
                    "UPDATE devices SET simulation_mode = ?1, sim_profile_id = ?2, sim_config = ?3 WHERE id = ?4",
                    rusqlite::params![sim_mode as i32, sim_profile_id, sim_config, device_id],
                )
                .context("Update device simulation failed")?;
            Ok(affected > 0)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_device_simulation(
        &self,
        device_id: &str,
    ) -> anyhow::Result<Option<(bool, Option<i64>, Option<String>)>> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let result = conn.query_row(
                "SELECT simulation_mode, sim_profile_id, sim_config FROM devices WHERE id = ?1",
                rusqlite::params![device_id],
                |row| {
                    let mode: i32 = row.get(0)?;
                    Ok((mode != 0, row.get(1)?, row.get(2)?))
                },
            );
            match result {
                Ok(row) => Ok(Some(row)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e).context("Get device simulation failed"),
            }
        })
        .await
        .context("Task join failed")?
    }

    // ── Script CRUD methods ──

    pub async fn list_scripts(&self) -> anyhow::Result<Vec<crate::models::Script>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, name, source, syntax_error, cpu_limit_ms, memory_limit_bytes, enabled, created_at, updated_at FROM scripts ORDER BY name")
                .context("Prepare list_scripts failed")?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(crate::models::Script {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        source: row.get(2)?,
                        syntax_error: row.get(3)?,
                        cpu_limit_ms: row.get(4)?,
                        memory_limit_bytes: row.get(5)?,
                        enabled: row.get::<_, i32>(6)? != 0,
                        created_at: row.get(7)?,
                        updated_at: row.get(8)?,
                    })
                })
                .context("Query list_scripts failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row parse list_scripts failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn get_script(&self, id: i64) -> anyhow::Result<Option<crate::models::Script>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, name, source, syntax_error, cpu_limit_ms, memory_limit_bytes, enabled, created_at, updated_at FROM scripts WHERE id = ?1")
                .context("Prepare get_script failed")?;
            let mut rows = stmt
                .query_map(rusqlite::params![id], |row| {
                    Ok(crate::models::Script {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        source: row.get(2)?,
                        syntax_error: row.get(3)?,
                        cpu_limit_ms: row.get(4)?,
                        memory_limit_bytes: row.get(5)?,
                        enabled: row.get::<_, i32>(6)? != 0,
                        created_at: row.get(7)?,
                        updated_at: row.get(8)?,
                    })
                })
                .context("Query get_script failed")?;
            match rows.next() {
                Some(row) => Ok(Some(row.context("Row parse get_script failed")?)),
                None => Ok(None),
            }
        })
        .await
        .context("Task join failed")?
    }

    pub async fn create_script(
        &self,
        name: &str,
        source: &str,
        syntax_error: Option<&str>,
        cpu_limit_ms: i64,
        memory_limit_bytes: i64,
    ) -> anyhow::Result<i64> {
        let conn = self.conn.clone();
        let name = name.to_string();
        let source = source.to_string();
        let syntax_error = syntax_error.map(|s| s.to_string());
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO scripts (name, source, syntax_error, cpu_limit_ms, memory_limit_bytes, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
                rusqlite::params![name, source, syntax_error, cpu_limit_ms, memory_limit_bytes, now, now],
            )
            .context("Insert script failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn update_script(
        &self,
        id: i64,
        name: Option<&str>,
        source: Option<&str>,
        syntax_error: Option<&str>,
        cpu_limit_ms: Option<i64>,
        memory_limit_bytes: Option<i64>,
        enabled: Option<bool>,
    ) -> anyhow::Result<bool> {
        let conn = self.conn.clone();
        let name = name.map(|s| s.to_string());
        let source = source.map(|s| s.to_string());
        let syntax_error = syntax_error.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            // Check existence
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM scripts WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get::<_, i32>(0),
                )
                .context("Check script existence failed")?
                > 0;
            if !exists {
                return Ok(false);
            }

            // Build dynamic SET clauses
            let mut set_clauses: Vec<String> = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(ref v) = name {
                set_clauses.push("name = ?".to_string());
                params.push(Box::new(v.clone()));
            }
            if let Some(ref v) = source {
                set_clauses.push("source = ?".to_string());
                params.push(Box::new(v.clone()));
            }
            if let Some(ref v) = syntax_error {
                set_clauses.push("syntax_error = ?".to_string());
                params.push(Box::new(v.clone()));
            }
            if let Some(v) = cpu_limit_ms {
                set_clauses.push("cpu_limit_ms = ?".to_string());
                params.push(Box::new(v));
            }
            if let Some(v) = memory_limit_bytes {
                set_clauses.push("memory_limit_bytes = ?".to_string());
                params.push(Box::new(v));
            }
            if let Some(v) = enabled {
                set_clauses.push("enabled = ?".to_string());
                params.push(Box::new(v as i32));
            }

            if set_clauses.is_empty() {
                return Ok(false);
            }

            // Always update updated_at
            let now = crate::timeutil::now_secs();
            set_clauses.push("updated_at = ?".to_string());
            params.push(Box::new(now));

            // Add WHERE id param
            params.push(Box::new(id));

            // Build parameter placeholders
            let placeholders: Vec<String> = (1..=params.len())
                .map(|i| format!("?{}", i))
                .collect();
            let placeholders_str = placeholders.join(", ");

            let sql = format!(
                "UPDATE scripts SET {} WHERE id = {}",
                set_clauses.join(", "),
                placeholders_str
            );

            // Recalculate: we need positional params, rebuild with proper numbering
            let mut set_parts: Vec<String> = Vec::new();
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            let mut idx = 1usize;

            if let Some(ref v) = name {
                set_parts.push(format!("name = ?{}", idx));
                param_values.push(Box::new(v.clone()));
                idx += 1;
            }
            if let Some(ref v) = source {
                set_parts.push(format!("source = ?{}", idx));
                param_values.push(Box::new(v.clone()));
                idx += 1;
            }
            if let Some(ref v) = syntax_error {
                set_parts.push(format!("syntax_error = ?{}", idx));
                param_values.push(Box::new(v.clone()));
                idx += 1;
            }
            if let Some(v) = cpu_limit_ms {
                set_parts.push(format!("cpu_limit_ms = ?{}", idx));
                param_values.push(Box::new(v));
                idx += 1;
            }
            if let Some(v) = memory_limit_bytes {
                set_parts.push(format!("memory_limit_bytes = ?{}", idx));
                param_values.push(Box::new(v));
                idx += 1;
            }
            if let Some(v) = enabled {
                set_parts.push(format!("enabled = ?{}", idx));
                param_values.push(Box::new(v as i32));
                idx += 1;
            }
            set_parts.push(format!("updated_at = ?{}", idx));
            param_values.push(Box::new(now));
            idx += 1;

            let sql = format!(
                "UPDATE scripts SET {} WHERE id = ?{}",
                set_parts.join(", "),
                idx
            );
            param_values.push(Box::new(id));

            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                param_values.iter().map(|p| p.as_ref()).collect();

            let affected = conn
                .execute(&sql, param_refs.as_slice())
                .context("Update script failed")?;
            Ok(affected > 0)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn delete_script(&self, id: i64) -> anyhow::Result<bool> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let affected = conn
                .execute("DELETE FROM scripts WHERE id = ?1", rusqlite::params![id])
                .context("Delete script failed")?;
            Ok(affected > 0)
        })
        .await
        .context("Task join failed")?
    }

    // ── Script Execution CRUD methods ──

    pub async fn create_script_execution(
        &self,
        script_id: i64,
        flow_id: Option<&str>,
        trigger_type: Option<&str>,
        duration_ms: i64,
        status: &str,
        error_message: Option<&str>,
        output: Option<&str>,
    ) -> anyhow::Result<i64> {
        let conn = self.conn.clone();
        let flow_id = flow_id.map(|s| s.to_string());
        let trigger_type = trigger_type.map(|s| s.to_string());
        let status = status.to_string();
        let error_message = error_message.map(|s| s.to_string());
        let output = output.map(|s| s.to_string());
        let now = crate::timeutil::now_secs();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO script_executions (script_id, flow_id, trigger_type, duration_ms, status, error_message, output, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![script_id, flow_id, trigger_type, duration_ms, status, error_message, output, now],
            )
            .context("Insert script_execution failed")?;
            Ok(conn.last_insert_rowid())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_script_executions(
        &self,
        script_id: i64,
        limit: i64,
    ) -> anyhow::Result<Vec<crate::models::ScriptExecution>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, script_id, flow_id, trigger_type, duration_ms, status, error_message, output, created_at FROM script_executions WHERE script_id = ?1 ORDER BY created_at DESC LIMIT ?2")
                .context("Prepare list_script_executions failed")?;
            let rows = stmt
                .query_map(rusqlite::params![script_id, limit], |row| {
                    Ok(crate::models::ScriptExecution {
                        id: row.get(0)?,
                        script_id: row.get(1)?,
                        flow_id: row.get(2)?,
                        trigger_type: row.get(3)?,
                        duration_ms: row.get(4)?,
                        status: row.get(5)?,
                        error_message: row.get(6)?,
                        output: row.get(7)?,
                        created_at: row.get(8)?,
                    })
                })
                .context("Query list_script_executions failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row parse list_script_executions failed")
        })
        .await
        .context("Task join failed")?
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

    #[tokio::test]
    async fn test_count_modbus_devices_empty() {
        let storage = Storage::new_in_memory().unwrap();
        let count = storage.count_modbus_devices().await.unwrap();
        assert_eq!(count, 0);
    }

    // ── Security audit log tests ──

    #[tokio::test]
    async fn test_insert_and_list_security_events() {
        let storage = Storage::new_in_memory().unwrap();

        // Insert events
        storage.insert_security_event(&crate::models::CreateSecurityEvent {
            event_type: "login_failed".into(),
            severity: "warning".into(),
            source_ip: Some("192.168.1.100".into()),
            user_agent: None,
            username: Some("admin".into()),
            details: Some("Invalid password (attempt 1)".into()),
        }).await.unwrap();

        storage.insert_security_event(&crate::models::CreateSecurityEvent {
            event_type: "auth_401".into(),
            severity: "info".into(),
            source_ip: Some("10.0.0.1".into()),
            user_agent: None,
            username: None,
            details: Some("Unauthorized request".into()),
        }).await.unwrap();

        storage.insert_security_event(&crate::models::CreateSecurityEvent {
            event_type: "config_change".into(),
            severity: "info".into(),
            source_ip: None,
            user_agent: None,
            username: Some("admin".into()),
            details: Some("Updated branding".into()),
        }).await.unwrap();

        // List all
        let events = storage.list_security_events(None, None, None, None, 100, 0).await.unwrap();
        assert_eq!(events.len(), 3);

        // Filter by event_type
        let login_events = storage.list_security_events(Some("login_failed"), None, None, None, 100, 0).await.unwrap();
        assert_eq!(login_events.len(), 1);
        assert_eq!(login_events[0].event_type, "login_failed");

        // Filter by severity
        let warnings = storage.list_security_events(None, Some("warning"), None, None, 100, 0).await.unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].severity, "warning");

        // Pagination
        let page = storage.list_security_events(None, None, None, None, 2, 0).await.unwrap();
        assert_eq!(page.len(), 2);
        let page2 = storage.list_security_events(None, None, None, None, 2, 2).await.unwrap();
        assert_eq!(page2.len(), 1);
    }

    #[tokio::test]
    async fn test_export_security_events_csv() {
        let storage = Storage::new_in_memory().unwrap();

        storage.insert_security_event(&crate::models::CreateSecurityEvent {
            event_type: "login_failed".into(),
            severity: "warning".into(),
            source_ip: Some("192.168.1.100".into()),
            user_agent: Some("Mozilla/5.0".into()),
            username: Some("admin".into()),
            details: Some("Invalid password".into()),
        }).await.unwrap();

        let csv = storage.export_security_events_csv(None, None).await.unwrap();

        // Check CSV header
        assert!(csv.starts_with("id,event_type,severity,source_ip,user_agent,username,details,created_at\n"));
        // Check data row
        assert!(csv.contains("login_failed"));
        assert!(csv.contains("warning"));
        assert!(csv.contains("192.168.1.100"));
        assert!(csv.contains("admin"));
    }

    #[tokio::test]
    async fn test_security_thresholds_crud() {
        let storage = Storage::new_in_memory().unwrap();

        // Get defaults
        let thresholds = storage.get_security_thresholds().await.unwrap();
        assert_eq!(thresholds.failed_login_threshold, 5);
        assert_eq!(thresholds.suspicious_401_threshold, 10);
        assert_eq!(thresholds.rate_limit_threshold, 100);

        // Update
        let updated = crate::models::SecurityThresholds {
            failed_login_threshold: 10,
            failed_login_window_secs: 120,
            suspicious_401_threshold: 20,
            suspicious_401_window_secs: 120,
            rate_limit_threshold: 200,
            rate_limit_window_secs: 30,
        };
        storage.update_security_thresholds(&updated).await.unwrap();

        // Verify
        let thresholds = storage.get_security_thresholds().await.unwrap();
        assert_eq!(thresholds.failed_login_threshold, 10);
        assert_eq!(thresholds.suspicious_401_threshold, 20);
        assert_eq!(thresholds.rate_limit_threshold, 200);
    }

    #[tokio::test]
    async fn test_security_events_append_only() {
        let storage = Storage::new_in_memory().unwrap();

        // Insert multiple events
        for i in 0..5 {
            storage.insert_security_event(&crate::models::CreateSecurityEvent {
                event_type: "auth_401".into(),
                severity: "warning".into(),
                source_ip: Some(format!("10.0.0.{}", i)),
                user_agent: None,
                username: None,
                details: None,
            }).await.unwrap();
        }

        // All events should be present (append-only, no deletion)
        let events = storage.list_security_events(None, None, None, None, 100, 0).await.unwrap();
        assert_eq!(events.len(), 5);
    }

    #[tokio::test]
    async fn test_security_events_csv_escapes_special_chars() {
        let storage = Storage::new_in_memory().unwrap();

        storage.insert_security_event(&crate::models::CreateSecurityEvent {
            event_type: "login_failed".into(),
            severity: "warning".into(),
            source_ip: Some("192.168.1.1".into()),
            user_agent: Some("Agent with \"quotes\"".into()),
            username: Some("user,test".into()),
            details: Some("Line1\nLine2".into()),
        }).await.unwrap();

        let csv = storage.export_security_events_csv(None, None).await.unwrap();
        assert!(csv.contains("user,test"));
        assert!(csv.contains("\\n"));
    }

    #[tokio::test]
    async fn test_count_modbus_devices_with_data() {
        let storage = Storage::new_in_memory().unwrap();
        storage.create_modbus_device(
            "dev_count_1",
            "Test Device",
            "192.168.1.1",
            502,
            1,
            1000,
            "big",
            &[],
        ).await.unwrap();
        let count = storage.count_modbus_devices().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_hub_site_crud() {
        let storage = Storage::new_in_memory().unwrap();

        // Create a site
        let id = storage.create_hub_site("Factory A", "https://factory-a.example.com", "secret-key-123", true).await.unwrap();
        assert!(id > 0);

        // List sites
        let sites = storage.list_hub_sites().await.unwrap();
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].name, "Factory A");
        assert_eq!(sites[0].url, "https://factory-a.example.com");
        assert_eq!(sites[0].enabled, true);
        assert!(sites[0].last_seen.is_none());

        // Update site
        storage.update_hub_site(id, Some("Factory A Updated"), None, None, None).await.unwrap();
        let sites = storage.list_hub_sites().await.unwrap();
        assert_eq!(sites[0].name, "Factory A Updated");

        // Update last_seen
        storage.update_hub_site_last_seen(id).await.unwrap();
        let sites = storage.list_hub_sites().await.unwrap();
        assert!(sites[0].last_seen.is_some());

        // Get enabled sites
        let enabled = storage.get_enabled_hub_sites().await.unwrap();
        assert_eq!(enabled.len(), 1);

        // Disable site
        storage.update_hub_site(id, None, None, None, Some(false)).await.unwrap();
        let enabled = storage.get_enabled_hub_sites().await.unwrap();
        assert_eq!(enabled.len(), 0);

        // Delete site
        storage.delete_hub_site(id).await.unwrap();
        let sites = storage.list_hub_sites().await.unwrap();
        assert_eq!(sites.len(), 0);
    }

    #[tokio::test]
    async fn test_hub_site_multiple_sites() {
        let storage = Storage::new_in_memory().unwrap();

        let id1 = storage.create_hub_site("Site 1", "https://site1.example.com", "key1", true).await.unwrap();
        let id2 = storage.create_hub_site("Site 2", "https://site2.example.com", "key2", false).await.unwrap();

        let all = storage.list_hub_sites().await.unwrap();
        assert_eq!(all.len(), 2);

        let enabled = storage.get_enabled_hub_sites().await.unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].id, id1);

        // Delete one
        storage.delete_hub_site(id1).await.unwrap();
        let all = storage.list_hub_sites().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, id2);
    }

    #[tokio::test]
    async fn test_downtime_event_crud() {
        let storage = Storage::new_in_memory().unwrap();

        // Create open event
        let id = storage.create_downtime_event("dev1", "faulted").await.unwrap();
        assert!(id > 0);

        // Verify open event
        let open = storage.get_open_downtime_event("dev1").await.unwrap();
        assert!(open.is_some());
        let open = open.unwrap();
        assert_eq!(open.device_id, "dev1");
        assert_eq!(open.state, "faulted");
        assert!(open.ended_at.is_none());

        // Close event
        storage.close_downtime_event(id, Some("Machine repaired")).await.unwrap();

        // Verify closed
        let open_after = storage.get_open_downtime_event("dev1").await.unwrap();
        assert!(open_after.is_none());

        // List events
        let events = storage.list_downtime_events(Some("dev1"), None, None).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason.as_deref(), Some("Machine repaired"));
        assert!(events[0].duration_secs.is_some());
        assert!(events[0].duration_secs.unwrap() >= 0);
    }

    #[tokio::test]
    async fn test_oee_metrics_calculation() {
        let storage = Storage::new_in_memory().unwrap();

        // Create a downtime event that represents 1 hour of downtime
        let id = storage.create_downtime_event("dev1", "idle").await.unwrap();
        storage.close_downtime_event(id, Some("Break")).await.unwrap();

        // Calculate OEE for a 8-hour window
        let now = crate::timeutil::now_secs();
        let from = now - 8 * 3600;
        let metrics = storage.calculate_oee("dev1", from, now).await.unwrap();

        assert_eq!(metrics.total_planned_secs, 8 * 3600);
        assert!(metrics.total_running_secs > 0);
        assert!(metrics.availability <= 1.0);
        assert!(metrics.availability >= 0.0);
        assert_eq!(metrics.performance, 1.0); // default
        assert_eq!(metrics.quality, 1.0); // default
        assert_eq!(metrics.oee, metrics.availability);
        assert!(metrics.downtime_count >= 1);
    }

    #[tokio::test]
    async fn test_shift_crud() {
        let storage = Storage::new_in_memory().unwrap();

        // Create shift
        let id = storage.create_shift("Morning", 6, 0, 8, 31).await.unwrap(); // Mon-Fri
        assert!(id > 0);

        // List shifts
        let shifts = storage.list_shifts().await.unwrap();
        assert_eq!(shifts.len(), 1);
        assert_eq!(shifts[0].name, "Morning");
        assert_eq!(shifts[0].start_hour, 6);
        assert_eq!(shifts[0].start_minute, 0);
        assert_eq!(shifts[0].duration_hours, 8);
        assert_eq!(shifts[0].days_mask, 31);

        // Delete shift
        storage.delete_shift(id).await.unwrap();
        let shifts = storage.list_shifts().await.unwrap();
        assert_eq!(shifts.len(), 0);
    }

    // ── Command & Control integration tests (v0.24.0) ──

    #[tokio::test]
    async fn test_command_crud_lifecycle() {
        let storage = Storage::new_in_memory().unwrap();

        // Insert
        let cmd = Command {
            id: "cmd_test1".to_string(),
            data_source_id: 1,
            device_id: 1,
            protocol: "modbus".to_string(),
            value: "100.5".to_string(),
            status: "pending".to_string(),
            policy: "immediate".to_string(),
            scheduled_at: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
            created_at: 1000,
            sent_at: None,
            acked_at: None,
            resolved_at: None,
            created_by: "test_user".to_string(),
        };
        storage.insert_command(&cmd).await.unwrap();

        // Get
        let fetched = storage.get_command("cmd_test1").await.unwrap().unwrap();
        assert_eq!(fetched.status, "pending");
        assert_eq!(fetched.value, "100.5");
        assert_eq!(fetched.created_by, "test_user");

        // Update to acked
        storage
            .update_command_status("cmd_test1", "acked", None, "system", None)
            .await
            .unwrap();
        let fetched = storage.get_command("cmd_test1").await.unwrap().unwrap();
        assert_eq!(fetched.status, "acked");
        assert!(fetched.acked_at.is_some());
        assert!(fetched.resolved_at.is_some());
    }

    #[tokio::test]
    async fn test_command_audit_trail() {
        let storage = Storage::new_in_memory().unwrap();

        let cmd = Command {
            id: "cmd_audit1".to_string(),
            data_source_id: 1,
            device_id: 1,
            protocol: "modbus".to_string(),
            value: "50".to_string(),
            status: "pending".to_string(),
            policy: "immediate".to_string(),
            scheduled_at: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
            created_at: 1000,
            sent_at: None,
            acked_at: None,
            resolved_at: None,
            created_by: "test_user".to_string(),
        };
        storage.insert_command(&cmd).await.unwrap();

        storage
            .update_command_status("cmd_audit1", "queued", Some("device offline"), "system", Some("retry"))
            .await
            .unwrap();
        storage
            .update_command_status("cmd_audit1", "acked", None, "system", None)
            .await
            .unwrap();

        let audit = storage.get_command_audit("cmd_audit1").await.unwrap();
        assert_eq!(audit.len(), 2);
        assert_eq!(audit[0].old_status, "pending");
        assert_eq!(audit[0].new_status, "queued");
        assert_eq!(audit[1].old_status, "queued");
        assert_eq!(audit[1].new_status, "acked");
    }

    #[tokio::test]
    async fn test_count_recent_writes() {
        let storage = Storage::new_in_memory().unwrap();
        let now = crate::timeutil::now_secs();

        for i in 0..5 {
            let cmd = Command {
                id: format!("cmd_rate{}", i),
                data_source_id: 42,
                device_id: 1,
                protocol: "modbus".to_string(),
                value: "10".to_string(),
                status: "acked".to_string(),
                policy: "immediate".to_string(),
                scheduled_at: None,
                retry_count: 0,
                max_retries: 3,
                error_message: None,
                created_at: now,
                sent_at: None,
                acked_at: None,
                resolved_at: None,
                created_by: "test".to_string(),
            };
            storage.insert_command(&cmd).await.unwrap();
        }

        let count = storage.count_recent_writes(42, 60).await.unwrap();
        assert_eq!(count, 5);

        // Different data source should be 0
        let count2 = storage.count_recent_writes(99, 60).await.unwrap();
        assert_eq!(count2, 0);
    }

    #[tokio::test]
    async fn test_list_commands_with_filters() {
        let storage = Storage::new_in_memory().unwrap();
        let now = crate::timeutil::now_secs();

        // Insert commands with different statuses
        for i in 0..3 {
            let cmd = Command {
                id: format!("cmd_list_acked_{}", i),
                data_source_id: 1,
                device_id: 1,
                protocol: "modbus".to_string(),
                value: "10".to_string(),
                status: "acked".to_string(),
                policy: "immediate".to_string(),
                scheduled_at: None,
                retry_count: 0,
                max_retries: 3,
                error_message: None,
                created_at: now,
                sent_at: None,
                acked_at: None,
                resolved_at: None,
                created_by: "test".to_string(),
            };
            storage.insert_command(&cmd).await.unwrap();
        }

        let failed_cmd = Command {
            id: "cmd_list_failed".to_string(),
            data_source_id: 2,
            device_id: 2,
            protocol: "modbus".to_string(),
            value: "20".to_string(),
            status: "failed".to_string(),
            policy: "immediate".to_string(),
            scheduled_at: None,
            retry_count: 3,
            max_retries: 3,
            error_message: Some("timeout".to_string()),
            created_at: now,
            sent_at: None,
            acked_at: None,
            resolved_at: None,
            created_by: "test".to_string(),
        };
        storage.insert_command(&failed_cmd).await.unwrap();

        // Filter by status
        let query = CommandQuery {
            status: Some("acked".to_string()),
            ..Default::default()
        };
        let results = storage.list_commands(&query).await.unwrap();
        assert_eq!(results.len(), 3);

        // Filter by device
        let query2 = CommandQuery {
            device: Some("2".to_string()),
            ..Default::default()
        };
        let results2 = storage.list_commands(&query2).await.unwrap();
        assert_eq!(results2.len(), 1);
        assert_eq!(results2[0].status, "failed");
    }

    #[tokio::test]
    async fn test_get_queued_commands() {
        let storage = Storage::new_in_memory().unwrap();

        // Insert a pending command
        let cmd = Command {
            id: "cmd_q1".to_string(),
            data_source_id: 1,
            device_id: 1,
            protocol: "modbus".to_string(),
            value: "10".to_string(),
            status: "pending".to_string(),
            policy: "immediate".to_string(),
            scheduled_at: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
            created_at: 1000,
            sent_at: None,
            acked_at: None,
            resolved_at: None,
            created_by: "test".to_string(),
        };
        storage.insert_command(&cmd).await.unwrap();

        // Insert a queued command
        let cmd2 = Command {
            id: "cmd_q2".to_string(),
            data_source_id: 1,
            device_id: 1,
            protocol: "modbus".to_string(),
            value: "20".to_string(),
            status: "queued".to_string(),
            policy: "immediate".to_string(),
            scheduled_at: None,
            retry_count: 1,
            max_retries: 3,
            error_message: None,
            created_at: 2000,
            sent_at: None,
            acked_at: None,
            resolved_at: None,
            created_by: "test".to_string(),
        };
        storage.insert_command(&cmd2).await.unwrap();

        let queued = storage.get_queued_commands().await.unwrap();
        assert_eq!(queued.len(), 2);
    }
}
