use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::models::{
    DataPoint, Device, ScreenLayoutRow, DataSource, Setting, TwinLayout, TwinConnection, User, UserShortcut,
    NavShortcut, Alarm, CanvasLayoutItem,
    ModbusDevice, ModbusRegister, ModbusRegisterInput, DeviceConfig,
    ScadaEquipment, ScadaConnection,
};

#[derive(Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database: {path}"))?;
        init_schema(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to open in-memory DB")?;
        init_schema(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
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

    // ── Users ──

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let conn = self.conn.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, username, password_hash, salt, COALESCE(role, 'user') FROM users WHERE username = ?1")
                .context("Prepare failed")?;
            let mut rows = stmt
                .query_map(rusqlite::params![username], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password_hash: row.get(2)?,
                        salt: row.get(3)?,
                        role: row.get(4)?,
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
                "INSERT INTO users (username, password_hash, salt, role) VALUES (?1, ?2, ?3, ?4)",
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
                .prepare("SELECT id, username, password_hash, salt, COALESCE(role, 'user') FROM users ORDER BY id")
                .context("Prepare failed")?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password_hash: row.get(2)?,
                        salt: row.get(3)?,
                        role: row.get(4)?,
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
                .prepare("SELECT id, username, password_hash, salt, COALESCE(role, 'user') FROM users WHERE id = ?1")
                .context("Prepare failed")?;
            let result = stmt
                .query_row(rusqlite::params![id], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password_hash: row.get(2)?,
                        salt: row.get(3)?,
                        role: row.get(4)?,
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

    pub async fn update_user_password(&self, id: i64, password_hash: &str) -> Result<()> {
        let conn = self.conn.clone();
        let password_hash = password_hash.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE users SET password_hash = ?1 WHERE id = ?2",
                rusqlite::params![password_hash, id],
            )
            .context("Update user password failed")?;
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
            let password = "admin";
            let password_hash = crate::api::auth::hash_password(password);

            self.create_user("admin", &password_hash, "", "admin").await?;

            tracing::info!("Seeded admin user");
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
            let now = chrono::Utc::now().timestamp();
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
            let now = chrono::Utc::now().timestamp();
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
                            triggered_at, resolved_at, acknowledged_at, acknowledged_by
                     FROM alarms
                     WHERE acknowledged_at IS NULL
                     ORDER BY triggered_at DESC LIMIT ?1",
                    vec![Box::new(limit)],
                ),
                Some(s) if s == "acknowledged" => (
                    "SELECT id, source_key, severity, value, threshold, message,
                            triggered_at, resolved_at, acknowledged_at, acknowledged_by
                     FROM alarms
                     WHERE acknowledged_at IS NOT NULL
                     ORDER BY triggered_at DESC LIMIT ?1",
                    vec![Box::new(limit)],
                ),
                _ => (
                    "SELECT id, source_key, severity, value, threshold, message,
                            triggered_at, resolved_at, acknowledged_at, acknowledged_by
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
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    pub async fn acknowledge_alarm(&self, id: i64, username: &str) -> Result<()> {
        let conn = self.conn.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = chrono::Utc::now().timestamp();
            conn.execute(
                "UPDATE alarms SET acknowledged_at = ?1, acknowledged_by = ?2 WHERE id = ?3",
                rusqlite::params![now, username, id],
            )
            .context("Acknowledge alarm failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
    }

    pub async fn acknowledge_all(&self, username: &str) -> Result<()> {
        let conn = self.conn.clone();
        let username = username.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = chrono::Utc::now().timestamp();
            conn.execute(
                "UPDATE alarms SET acknowledged_at = ?1, acknowledged_by = ?2
                 WHERE acknowledged_at IS NULL",
                rusqlite::params![now, username],
            )
            .context("Acknowledge all alarms failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
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
            let now = chrono::Utc::now().timestamp();
            conn.execute(
                "INSERT INTO push_subscriptions (user, endpoint, p256dh, auth_key, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![user, endpoint, p256dh, auth_key, now],
            )
            .context("Insert push subscription failed")
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
            acknowledged_by TEXT
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
            created_at INTEGER NOT NULL
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
        let now = chrono::Utc::now().timestamp();

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
                        "INSERT OR IGNORE INTO modbus_registers (device_id, name, address, register_type, scale, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![device.id, reg.name, reg.address, reg_type, reg.scale, i as i64],
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
        let registers: Vec<(String, u16, String, f64)> = registers.iter().map(|r| {
            (r.name.clone(), r.address, r.register_type.clone(), r.scale)
        }).collect();
        let now = chrono::Utc::now().timestamp();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;

            tx.execute(
                "INSERT INTO modbus_devices (id, name, ip, port, unit_id, interval_ms, endianness, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8)",
                rusqlite::params![id, name, ip, port, unit_id, interval_ms, endianness, now],
            ).context("Insert modbus device failed")?;

            for (i, (rname, addr, rtype, scale)) in registers.iter().enumerate() {
                tx.execute(
                    "INSERT INTO modbus_registers (device_id, name, address, register_type, scale, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![id, rname, addr, rtype, scale, i as i64],
                ).context("Insert modbus register failed")?;
            }

            tx.commit().context("Commit failed")
        }).await.context("Task join failed")?
    }

    pub async fn update_modbus_device(&self, id: &str, fields: &serde_json::Value) -> Result<()> {
        let conn = self.conn.clone();
        let id = id.to_string();
        let now = chrono::Utc::now().timestamp();
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
                "SELECT id, device_id, name, address, register_type, scale, sort_order FROM modbus_registers WHERE device_id = ?1 ORDER BY sort_order"
            ).context("Prepare failed")?;
            let rows = stmt.query_map(rusqlite::params![device_id], |row| {
                Ok(ModbusRegister {
                    id: row.get(0)?,
                    device_id: row.get(1)?,
                    name: row.get(2)?,
                    address: row.get(3)?,
                    register_type: row.get(4)?,
                    scale: row.get(5)?,
                    sort_order: row.get(6)?,
                })
            }).context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>().context("Row mapping failed")
        }).await.context("Task join failed")?
    }

    pub async fn replace_modbus_registers(&self, device_id: &str, registers: &[ModbusRegisterInput]) -> Result<()> {
        let conn = self.conn.clone();
        let device_id = device_id.to_string();
        let registers: Vec<(String, u16, String, f64)> = registers.iter().map(|r| {
            (r.name.clone(), r.address, r.register_type.clone(), r.scale)
        }).collect();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let tx = conn.unchecked_transaction().context("Begin tx failed")?;
            tx.execute("DELETE FROM modbus_registers WHERE device_id = ?1", rusqlite::params![device_id])
                .context("Delete registers failed")?;
            for (i, (rname, addr, rtype, scale)) in registers.iter().enumerate() {
                tx.execute(
                    "INSERT INTO modbus_registers (device_id, name, address, register_type, scale, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![device_id, rname, addr, rtype, scale, i as i64],
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
}
