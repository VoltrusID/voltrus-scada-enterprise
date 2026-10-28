//! The database side of hot standby: the persistent node record, the change
//! feed and its application.
//!
//! # Mechanism
//!
//! Replication is **logical and trigger driven**, not WAL or page shipping.
//! SQLite has no replication API: WAL frames only make sense against the exact
//! base file, cannot be applied under a live connection, and would tie the
//! standby to the page layout of one SQLite build. A change feed at the row
//! level instead has these properties:
//!
//! * it is table-generic. Every table the schema has, now or after a later
//!   migration (historian packed blocks, new config tables), is replicated
//!   without code here. Compaction that deletes raw rows and writes packed rows
//!   replicates as exactly those row deletes and inserts;
//! * it survives a schema or SQLite version difference between the nodes;
//! * it runs next to the normal write path: the core writes through its one
//!   connection, `AFTER INSERT/UPDATE/DELETE` triggers append `(table, key)` to
//!   `ha_changelog` in the same transaction, so the log is exactly as durable as
//!   the data and a crash never loses a logged-but-unwritten change.
//!
//! The log records *which* row changed, not its contents. When a batch is
//! built, the current row is read and shipped as a full image (or as a delete
//! if it is gone). Applying an image is `INSERT OR REPLACE`, so replay is
//! idempotent and repeated changes to one row collapse into one image. The
//! batch is read in one snapshot, so it is consistent.
//!
//! Applying a batch on the standby runs with the `ha_guard.mute` flag set
//! inside the same transaction, so the standby's own triggers stay silent.

use std::collections::{BTreeMap, HashMap};

use anyhow::{Context, Result, bail};
use rusqlite::types::{Value, ValueRef};
use rusqlite::{Connection, OptionalExtension, params};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Tables that are never replicated: the replication bookkeeping itself and
/// the machine-bound licence activation (each node holds its own).
const NODE_LOCAL_TABLES: &[&str] = &["license_activation"];

/// Whether a table's rows replicate.
pub fn is_replicated(name: &str) -> bool {
    !(name.starts_with("sqlite_") || name.starts_with("ha_") || NODE_LOCAL_TABLES.contains(&name))
}

pub const OP_UPSERT: i64 = 1;
pub const OP_DELETE: i64 = 2;

// ── values on the wire ──

/// One SQLite value. Serialised as the plain MessagePack scalar it is.
#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Null,
    Int(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl Cell {
    fn from_ref(v: ValueRef<'_>) -> Cell {
        match v {
            ValueRef::Null => Cell::Null,
            ValueRef::Integer(i) => Cell::Int(i),
            ValueRef::Real(f) => Cell::Real(f),
            ValueRef::Text(t) => Cell::Text(String::from_utf8_lossy(t).into_owned()),
            ValueRef::Blob(b) => Cell::Blob(b.to_vec()),
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Cell::Null => Value::Null,
            Cell::Int(i) => Value::Integer(*i),
            Cell::Real(f) => Value::Real(*f),
            Cell::Text(t) => Value::Text(t.clone()),
            Cell::Blob(b) => Value::Blob(b.clone()),
        }
    }

    fn from_json(v: &serde_json::Value) -> Result<Cell> {
        Ok(match v {
            serde_json::Value::Null => Cell::Null,
            serde_json::Value::Number(n) => match n.as_i64() {
                Some(i) => Cell::Int(i),
                None => Cell::Real(n.as_f64().context("key number out of range")?),
            },
            serde_json::Value::String(s) => Cell::Text(s.clone()),
            serde_json::Value::Bool(b) => Cell::Int(*b as i64),
            other => bail!("unsupported key value in change log: {other}"),
        })
    }
}

impl Serialize for Cell {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Cell::Null => s.serialize_unit(),
            Cell::Int(i) => s.serialize_i64(*i),
            Cell::Real(f) => s.serialize_f64(*f),
            Cell::Text(t) => s.serialize_str(t),
            Cell::Blob(b) => s.serialize_bytes(b),
        }
    }
}

impl<'de> Deserialize<'de> for Cell {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct CellVisitor;
        impl<'de> Visitor<'de> for CellVisitor {
            type Value = Cell;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a SQLite value")
            }
            fn visit_unit<E: de::Error>(self) -> Result<Cell, E> {
                Ok(Cell::Null)
            }
            fn visit_none<E: de::Error>(self) -> Result<Cell, E> {
                Ok(Cell::Null)
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Cell, E> {
                Ok(Cell::Int(v))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Cell, E> {
                i64::try_from(v)
                    .map(Cell::Int)
                    .map_err(|_| E::custom("integer out of range"))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Cell, E> {
                Ok(Cell::Real(v))
            }
            fn visit_f32<E: de::Error>(self, v: f32) -> Result<Cell, E> {
                Ok(Cell::Real(v as f64))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Cell, E> {
                Ok(Cell::Text(v.to_owned()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Cell, E> {
                Ok(Cell::Text(v))
            }
            fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Cell, E> {
                Ok(Cell::Blob(v.to_vec()))
            }
            fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Cell, E> {
                Ok(Cell::Blob(v))
            }
        }
        d.deserialize_any(CellVisitor)
    }
}

// ── schema and node record ──

/// Create the replication tables. Idempotent.
pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS ha_node (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            node_id TEXT NOT NULL,
            role TEXT NOT NULL,
            epoch INTEGER NOT NULL,
            applied_seq INTEGER NOT NULL DEFAULT 0,
            needs_full_sync INTEGER NOT NULL DEFAULT 0,
            peer_running TEXT NOT NULL DEFAULT '[]',
            last_failover_at INTEGER,
            last_failover_reason TEXT,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS ha_guard (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            mute INTEGER NOT NULL
        );
        INSERT OR IGNORE INTO ha_guard (id, mute) VALUES (1, 0);
        UPDATE ha_guard SET mute = 0 WHERE id = 1;
        CREATE TABLE IF NOT EXISTS ha_changelog (
            seq INTEGER PRIMARY KEY AUTOINCREMENT,
            tbl TEXT NOT NULL,
            op INTEGER NOT NULL,
            pk TEXT NOT NULL
        );",
    )
    .context("Failed to create the HA tables")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredRole {
    Primary,
    Standby,
}

impl StoredRole {
    fn as_str(self) -> &'static str {
        match self {
            StoredRole::Primary => "primary",
            StoredRole::Standby => "standby",
        }
    }
}

/// What this node remembers across restarts.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeRecord {
    pub node_id: String,
    pub role: StoredRole,
    /// The term this node last held or last followed. Only ever grows.
    pub epoch: u64,
    /// Standby: the last change of the primary's log it applied, in the
    /// primary's sequence space of `epoch`.
    pub applied_seq: u64,
    /// The data set is not a clean copy of the primary's: a full sync has to
    /// run before this node may follow incrementally or take over.
    pub needs_full_sync: bool,
    /// The device pollers the primary reported running, so a takeover starts
    /// the same ones.
    pub peer_running: Vec<String>,
    pub last_failover_at: Option<i64>,
    pub last_failover_reason: Option<String>,
}

pub fn load_node(conn: &Connection) -> Result<Option<NodeRecord>> {
    conn.query_row(
        "SELECT node_id, role, epoch, applied_seq, needs_full_sync, peer_running,
                last_failover_at, last_failover_reason
         FROM ha_node WHERE id = 1",
        [],
        |r| {
            let role: String = r.get(1)?;
            let running: String = r.get(5)?;
            Ok(NodeRecord {
                node_id: r.get(0)?,
                role: if role == "primary" {
                    StoredRole::Primary
                } else {
                    StoredRole::Standby
                },
                epoch: r.get::<_, i64>(2)? as u64,
                applied_seq: r.get::<_, i64>(3)? as u64,
                needs_full_sync: r.get::<_, i64>(4)? != 0,
                peer_running: serde_json::from_str(&running).unwrap_or_default(),
                last_failover_at: r.get(6)?,
                last_failover_reason: r.get(7)?,
            })
        },
    )
    .optional()
    .context("Failed to read the HA node record")
}

pub fn save_node(conn: &Connection, n: &NodeRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO ha_node (id, node_id, role, epoch, applied_seq, needs_full_sync,
                              peer_running, last_failover_at, last_failover_reason, updated_at)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
            node_id = excluded.node_id, role = excluded.role, epoch = excluded.epoch,
            applied_seq = excluded.applied_seq, needs_full_sync = excluded.needs_full_sync,
            peer_running = excluded.peer_running, last_failover_at = excluded.last_failover_at,
            last_failover_reason = excluded.last_failover_reason, updated_at = excluded.updated_at",
        params![
            n.node_id,
            n.role.as_str(),
            n.epoch as i64,
            n.applied_seq as i64,
            n.needs_full_sync as i64,
            serde_json::to_string(&n.peer_running)?,
            n.last_failover_at,
            n.last_failover_reason,
            voltrus::timeutil::now_secs(),
        ],
    )
    .context("Failed to save the HA node record")?;
    Ok(())
}

// ── table inspection ──

/// What replication needs to know about one table.
#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    /// Replicated columns. For a table without a usable primary key the first
    /// entry is `rowid`.
    pub cols: Vec<String>,
    /// Key columns used to address a row. `["rowid"]` for tables keyed by rowid.
    pub key: Vec<String>,
    pub keyed_by_rowid: bool,
    pub has_rowid: bool,
    pub create_sql: String,
    pub index_sql: Vec<String>,
}

pub fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

fn quote_literal(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// Inspect one table. `None` when it is not a plain table replication can address.
pub fn table_schema(conn: &Connection, name: &str) -> Result<Option<TableSchema>> {
    let create_sql: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [name],
            |r| r.get(0),
        )
        .optional()?;
    let Some(create_sql) = create_sql else {
        return Ok(None);
    };
    if create_sql.to_uppercase().starts_with("CREATE VIRTUAL") {
        return Ok(None);
    }
    let has_rowid = !create_sql.to_uppercase().contains("WITHOUT ROWID");

    let mut stmt = conn.prepare(&format!("PRAGMA table_xinfo({})", quote_ident(name)))?;
    let mut cols = Vec::new();
    let mut pk: Vec<(i64, String, String)> = Vec::new();
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(5)?,
            r.get::<_, i64>(6)?,
        ))
    })?;
    for row in rows {
        let (col, ty, pk_pos, hidden) = row?;
        if hidden != 0 {
            continue; // generated or virtual-table column
        }
        if pk_pos > 0 {
            pk.push((pk_pos, col.clone(), ty));
        }
        cols.push(col);
    }
    pk.sort();
    let blob_key = pk.iter().any(|(_, _, ty)| ty.to_uppercase().contains("BLOB"));
    let keyed_by_rowid = pk.is_empty() || (blob_key && has_rowid);
    if keyed_by_rowid && !has_rowid {
        return Ok(None); // WITHOUT ROWID with no usable key cannot exist; blob keys are unsupported
    }
    let key = if keyed_by_rowid {
        vec!["rowid".to_string()]
    } else {
        pk.into_iter().map(|(_, c, _)| c).collect()
    };
    if keyed_by_rowid {
        cols.insert(0, "rowid".to_string());
    }

    let mut idx = conn.prepare(
        "SELECT sql FROM sqlite_master WHERE type = 'index' AND tbl_name = ?1 AND sql IS NOT NULL",
    )?;
    let index_sql = idx
        .query_map([name], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(Some(TableSchema {
        name: name.to_string(),
        cols,
        key,
        keyed_by_rowid,
        has_rowid,
        create_sql,
        index_sql,
    }))
}

/// Every replicated table in the database.
pub fn table_schemas(conn: &Connection) -> Result<BTreeMap<String, TableSchema>> {
    let mut stmt =
        conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")?;
    let names = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut out = BTreeMap::new();
    for name in names {
        if !is_replicated(&name) {
            continue;
        }
        match table_schema(conn, &name)? {
            Some(s) => {
                out.insert(name, s);
            }
            None => tracing::warn!("HA: table {name} cannot be replicated and is skipped"),
        }
    }
    Ok(out)
}

// ── triggers ──

fn key_expr(prefix: &str, s: &TableSchema) -> String {
    let parts: Vec<String> = s
        .key
        .iter()
        .map(|c| {
            if c == "rowid" && s.keyed_by_rowid {
                format!("{prefix}.rowid")
            } else {
                format!("{prefix}.{}", quote_ident(c))
            }
        })
        .collect();
    format!("json_array({})", parts.join(", "))
}

fn trigger_sql(s: &TableSchema) -> [(String, String); 3] {
    let t = quote_ident(&s.name);
    let lit = quote_literal(&s.name);
    let guard = "WHEN (SELECT mute FROM ha_guard WHERE id = 1) = 0";
    let changed = s
        .key
        .iter()
        .map(|c| {
            let col = if c == "rowid" && s.keyed_by_rowid {
                "rowid".to_string()
            } else {
                quote_ident(c)
            };
            format!("OLD.{col} IS NOT NEW.{col}")
        })
        .collect::<Vec<_>>()
        .join(" OR ");
    let name = |suffix: &str| quote_ident(&format!("ha_trg_{}_{suffix}", s.name));
    [
        (
            name("i"),
            format!(
                "CREATE TRIGGER IF NOT EXISTS {} AFTER INSERT ON {t} {guard}
                 BEGIN INSERT INTO ha_changelog (tbl, op, pk) VALUES ({lit}, {OP_UPSERT}, {}); END",
                name("i"),
                key_expr("NEW", s)
            ),
        ),
        (
            name("u"),
            format!(
                "CREATE TRIGGER IF NOT EXISTS {} AFTER UPDATE ON {t} {guard}
                 BEGIN
                   INSERT INTO ha_changelog (tbl, op, pk)
                     SELECT {lit}, {OP_DELETE}, {} WHERE {changed};
                   INSERT INTO ha_changelog (tbl, op, pk) VALUES ({lit}, {OP_UPSERT}, {});
                 END",
                name("u"),
                key_expr("OLD", s),
                key_expr("NEW", s)
            ),
        ),
        (
            name("d"),
            format!(
                "CREATE TRIGGER IF NOT EXISTS {} AFTER DELETE ON {t} {guard}
                 BEGIN INSERT INTO ha_changelog (tbl, op, pk) VALUES ({lit}, {OP_DELETE}, {}); END",
                name("d"),
                key_expr("OLD", s)
            ),
        ),
    ]
}

/// Make sure every replicated table has its three capture triggers. Cheap to
/// call again: it also covers tables created after the first call. Returns the
/// number of triggers created.
pub fn install_triggers(conn: &Connection) -> Result<usize> {
    let existing: std::collections::HashSet<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger' AND name LIKE 'ha_trg_%'")?;
        stmt.query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<_, _>>()?
    };
    let mut created = 0;
    for schema in table_schemas(conn)?.values() {
        for (quoted, sql) in trigger_sql(schema) {
            let bare = quoted.trim_matches('"').replace("\"\"", "\"");
            if !existing.contains(&bare) {
                conn.execute_batch(&sql)
                    .with_context(|| format!("Failed to create trigger {quoted}"))?;
                created += 1;
            }
        }
    }
    Ok(created)
}

/// Remove every capture trigger, the log and the guard. Used when HA is turned
/// off, so the log does not grow without a consumer.
pub fn teardown(conn: &Connection) -> Result<()> {
    let names: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger' AND name LIKE 'ha_trg_%'")?;
        stmt.query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<_, _>>()?
    };
    for n in names {
        conn.execute_batch(&format!("DROP TRIGGER IF EXISTS {}", quote_ident(&n)))?;
    }
    conn.execute_batch("DROP TABLE IF EXISTS ha_changelog; DROP TABLE IF EXISTS ha_guard;")?;
    Ok(())
}

pub fn has_triggers(conn: &Connection) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'trigger' AND name LIKE 'ha_trg_%')",
        [],
        |r| r.get::<_, i64>(0),
    )? != 0)
}

// ── the change log ──

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub seq: u64,
    pub tbl: String,
    pub op: i64,
    pub pk: String,
}

/// Highest sequence number ever assigned.
pub fn max_seq(conn: &Connection) -> Result<u64> {
    Ok(conn
        .query_row(
            "SELECT COALESCE((SELECT seq FROM sqlite_sequence WHERE name = 'ha_changelog'), 0)",
            [],
            |r| r.get::<_, i64>(0),
        )? as u64)
}

/// A standby whose position is at least this can continue incrementally.
pub fn log_floor(conn: &Connection) -> Result<u64> {
    let max = max_seq(conn)?;
    let min: Option<i64> =
        conn.query_row("SELECT MIN(seq) FROM ha_changelog", [], |r| r.get(0))?;
    Ok(match min {
        Some(m) => (m as u64).saturating_sub(1),
        None => max,
    })
}

pub fn read_log(conn: &Connection, after: u64, limit: usize) -> Result<Vec<LogEntry>> {
    let mut stmt = conn.prepare_cached(
        "SELECT seq, tbl, op, pk FROM ha_changelog WHERE seq > ?1 ORDER BY seq LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![after as i64, limit as i64], |r| {
        Ok(LogEntry {
            seq: r.get::<_, i64>(0)? as u64,
            tbl: r.get(1)?,
            op: r.get(2)?,
            pk: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Drop acknowledged entries.
pub fn prune_log(conn: &Connection, through: u64) -> Result<usize> {
    Ok(conn.execute("DELETE FROM ha_changelog WHERE seq <= ?1", [through as i64])?)
}

/// Keep at most `max_entries` of the newest entries. A standby that is behind
/// the new floor needs a full sync. Returns the number of entries dropped.
pub fn trim_log(conn: &Connection, max_entries: u64) -> Result<usize> {
    let max = max_seq(conn)?;
    if max <= max_entries {
        return Ok(0);
    }
    prune_log(conn, max - max_entries)
}

/// Forget every logged change: a node that just took over starts a new term.
pub fn clear_log(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM ha_changelog", [])?;
    Ok(())
}

/// Continue numbering after `seq`: the next logged change gets `seq + 1`. A
/// node that takes over from a primary that stopped cleanly continues its
/// numbering, so the old primary can follow it without a full sync.
pub fn set_next_seq(conn: &Connection, seq: u64) -> Result<()> {
    let updated = conn.execute(
        "UPDATE sqlite_sequence SET seq = ?1 WHERE name = 'ha_changelog'",
        [seq as i64],
    )?;
    if updated == 0 {
        conn.execute(
            "INSERT INTO sqlite_sequence (name, seq) VALUES ('ha_changelog', ?1)",
            [seq as i64],
        )?;
    }
    Ok(())
}

// ── change batches ──

/// The changes of one table in a batch.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TableChanges {
    pub name: String,
    pub cols: Vec<String>,
    pub key: Vec<String>,
    /// Full row images, columns in `cols` order.
    pub rows: Vec<Vec<Cell>>,
    /// Deleted keys, key columns in `key` order.
    pub deletes: Vec<Vec<Cell>>,
    /// Deleted runs `(lo, hi)` of a single integer key. Bulk deletes (retention,
    /// compaction) are runs, so they cost a few bytes instead of one key per row.
    pub delete_runs: Vec<(i64, i64)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Batch {
    pub seq_from: u64,
    pub seq_to: u64,
    pub tables: Vec<TableChanges>,
}

impl Batch {
    pub fn change_count(&self) -> usize {
        self.tables
            .iter()
            .map(|t| {
                t.rows.len()
                    + t.deletes.len()
                    + t.delete_runs
                        .iter()
                        .map(|(lo, hi)| (hi - lo + 1) as usize)
                        .sum::<usize>()
            })
            .sum()
    }
}

/// Turn log entries into a batch of row images and deletes. Reads on `conn`,
/// which should be inside one read transaction so the images are consistent.
/// A key changed several times costs one image.
pub fn build_batch(
    conn: &Connection,
    schemas: &BTreeMap<String, TableSchema>,
    entries: &[LogEntry],
) -> Result<Batch> {
    let (seq_from, seq_to) = match (entries.first(), entries.last()) {
        (Some(a), Some(b)) => (a.seq, b.seq),
        _ => return Ok(Batch::default()),
    };

    // The last operation per key wins, in the order keys first appeared.
    let mut order: Vec<(String, String)> = Vec::new();
    let mut last: HashMap<(String, String), i64> = HashMap::new();
    for e in entries {
        let k = (e.tbl.clone(), e.pk.clone());
        if last.insert(k.clone(), e.op).is_none() {
            order.push(k);
        }
    }

    let mut by_table: BTreeMap<String, TableChanges> = BTreeMap::new();
    let mut int_deletes: HashMap<String, Vec<i64>> = HashMap::new();
    for (tbl, pk_json) in order {
        let op = last[&(tbl.clone(), pk_json.clone())];
        let Some(schema) = schemas.get(&tbl) else {
            continue; // table dropped or excluded since the entry was logged
        };
        let key_vals = parse_key(&pk_json, schema.key.len())?;
        let entry = by_table.entry(tbl.clone()).or_insert_with(|| TableChanges {
            name: tbl.clone(),
            cols: schema.cols.clone(),
            key: schema.key.clone(),
            ..Default::default()
        });
        let mut is_delete = op == OP_DELETE;
        if !is_delete {
            match fetch_row(conn, schema, &key_vals)? {
                Some(row) => entry.rows.push(row),
                None => is_delete = true, // deleted after it was logged
            }
        }
        if is_delete {
            match key_vals.as_slice() {
                [Cell::Int(i)] => int_deletes.entry(tbl.clone()).or_default().push(*i),
                _ => entry.deletes.push(key_vals),
            }
        }
    }
    for (tbl, mut ids) in int_deletes {
        ids.sort_unstable();
        ids.dedup();
        let entry = by_table.get_mut(&tbl).expect("table seen above");
        let mut iter = ids.into_iter();
        if let Some(first) = iter.next() {
            let (mut lo, mut hi) = (first, first);
            for i in iter {
                if i == hi + 1 {
                    hi = i;
                } else {
                    entry.delete_runs.push((lo, hi));
                    lo = i;
                    hi = i;
                }
            }
            entry.delete_runs.push((lo, hi));
        }
    }
    Ok(Batch {
        seq_from,
        seq_to,
        tables: by_table.into_values().collect(),
    })
}

fn parse_key(pk_json: &str, width: usize) -> Result<Vec<Cell>> {
    let v: serde_json::Value = serde_json::from_str(pk_json).context("bad key in change log")?;
    let arr = v.as_array().context("change log key is not an array")?;
    anyhow::ensure!(arr.len() == width, "change log key has the wrong width");
    arr.iter().map(Cell::from_json).collect()
}

fn select_sql(s: &TableSchema) -> String {
    let cols = s
        .cols
        .iter()
        .map(|c| if c == "rowid" && s.keyed_by_rowid { "rowid".to_string() } else { quote_ident(c) })
        .collect::<Vec<_>>()
        .join(", ");
    let wher = s
        .key
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let col = if c == "rowid" && s.keyed_by_rowid { "rowid".to_string() } else { quote_ident(c) };
            format!("{col} = ?{}", i + 1)
        })
        .collect::<Vec<_>>()
        .join(" AND ");
    format!("SELECT {cols} FROM {} WHERE {wher}", quote_ident(&s.name))
}

fn fetch_row(conn: &Connection, s: &TableSchema, key: &[Cell]) -> Result<Option<Vec<Cell>>> {
    let mut stmt = conn.prepare_cached(&select_sql(s))?;
    let vals: Vec<Value> = key.iter().map(Cell::to_value).collect();
    let n = s.cols.len();
    let row = stmt
        .query_row(rusqlite::params_from_iter(vals.iter()), |r| {
            (0..n)
                .map(|i| r.get_ref(i).map(Cell::from_ref))
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .optional()?;
    Ok(row)
}

// ── applying changes ──

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ApplyStats {
    pub upserts: usize,
    pub deletes: usize,
}

/// Where the standby stands after a transaction, persisted with it.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub epoch: u64,
    pub applied_seq: u64,
    pub needs_full_sync: bool,
}

/// Check a table name and its columns against the local schema before they are
/// put into SQL: the peer is authenticated, but a name from the wire is still
/// never trusted to be an identifier.
fn local_schema<'a>(
    conn: &Connection,
    cache: &'a mut HashMap<String, TableSchema>,
    t: &TableChanges,
) -> Result<&'a TableSchema> {
    if !cache.contains_key(&t.name) {
        anyhow::ensure!(is_replicated(&t.name), "table {} is not replicated", t.name);
        let s = table_schema(conn, &t.name)?
            .with_context(|| format!("table {} does not exist on this node", t.name))?;
        cache.insert(t.name.clone(), s);
    }
    let s = &cache[&t.name];
    for c in &t.cols {
        anyhow::ensure!(
            s.cols.contains(c),
            "column {c} of table {} does not exist on this node (different schema versions?)",
            t.name
        );
    }
    anyhow::ensure!(
        t.key.iter().all(|k| s.key.contains(k) || s.cols.contains(k)),
        "key of table {} does not match this node",
        t.name
    );
    Ok(&cache[&t.name])
}

fn apply_table(
    tx: &rusqlite::Transaction<'_>,
    cache: &mut HashMap<String, TableSchema>,
    t: &TableChanges,
    stats: &mut ApplyStats,
) -> Result<()> {
    let schema = local_schema(tx, cache, t)?.clone();
    let key_col = |c: &String| {
        if c == "rowid" && schema.keyed_by_rowid { "rowid".to_string() } else { quote_ident(c) }
    };

    if !t.delete_runs.is_empty() {
        anyhow::ensure!(t.key.len() == 1, "delete runs need a single-column key");
        let sql = format!(
            "DELETE FROM {} WHERE {} BETWEEN ?1 AND ?2",
            quote_ident(&t.name),
            key_col(&t.key[0])
        );
        let mut stmt = tx.prepare_cached(&sql)?;
        for (lo, hi) in &t.delete_runs {
            stats.deletes += stmt.execute(params![lo, hi])?;
        }
    }
    if !t.deletes.is_empty() {
        let wher = t
            .key
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{} = ?{}", key_col(c), i + 1))
            .collect::<Vec<_>>()
            .join(" AND ");
        let sql = format!("DELETE FROM {} WHERE {wher}", quote_ident(&t.name));
        let mut stmt = tx.prepare_cached(&sql)?;
        for key in &t.deletes {
            anyhow::ensure!(key.len() == t.key.len(), "delete key has the wrong width");
            let vals: Vec<Value> = key.iter().map(Cell::to_value).collect();
            stats.deletes += stmt.execute(rusqlite::params_from_iter(vals.iter()))?;
        }
    }
    if !t.rows.is_empty() {
        let cols = t.cols.iter().map(&key_col).collect::<Vec<_>>().join(", ");
        let marks = (1..=t.cols.len())
            .map(|i| format!("?{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "INSERT OR REPLACE INTO {} ({cols}) VALUES ({marks})",
            quote_ident(&t.name)
        );
        let mut stmt = tx.prepare_cached(&sql)?;
        for row in &t.rows {
            anyhow::ensure!(row.len() == t.cols.len(), "row image has the wrong width");
            let vals: Vec<Value> = row.iter().map(Cell::to_value).collect();
            stmt.execute(rusqlite::params_from_iter(vals.iter()))?;
            stats.upserts += 1;
        }
    }
    Ok(())
}

/// Apply changes in one transaction with the standby's own triggers muted,
/// and record the new position in the same transaction.
pub fn apply_changes(
    conn: &mut Connection,
    tables: &[TableChanges],
    position: Option<Position>,
) -> Result<ApplyStats> {
    with_foreign_keys_off(conn, |conn| apply_changes_inner(conn, tables, position))
}

/// The core turns foreign key enforcement on after its migrations. Replicated
/// rows arrive in no particular parent-first order and a parent's deletes are
/// replicated as deletes of its children already (the capture triggers log
/// cascaded deletes), so enforcement during apply would only reject valid
/// data. The pragma cannot change inside a transaction, hence the wrapper.
fn with_foreign_keys_off<T>(
    conn: &mut Connection,
    f: impl FnOnce(&mut Connection) -> Result<T>,
) -> Result<T> {
    let was: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0))?;
    if was != 0 {
        conn.execute_batch("PRAGMA foreign_keys = OFF")?;
    }
    let result = f(conn);
    if was != 0 {
        conn.execute_batch("PRAGMA foreign_keys = ON")?;
    }
    result
}

fn apply_changes_inner(
    conn: &mut Connection,
    tables: &[TableChanges],
    position: Option<Position>,
) -> Result<ApplyStats> {
    let tx = conn.transaction().context("Failed to begin the apply transaction")?;
    tx.execute("UPDATE ha_guard SET mute = 1 WHERE id = 1", [])?;
    let mut cache = HashMap::new();
    let mut stats = ApplyStats::default();
    for t in tables {
        apply_table(&tx, &mut cache, t, &mut stats)?;
    }
    if let Some(p) = position {
        tx.execute(
            "UPDATE ha_node SET epoch = ?1, applied_seq = ?2, needs_full_sync = ?3, updated_at = ?4
             WHERE id = 1",
            params![
                p.epoch as i64,
                p.applied_seq as i64,
                p.needs_full_sync as i64,
                voltrus::timeutil::now_secs()
            ],
        )?;
    }
    tx.execute("UPDATE ha_guard SET mute = 0 WHERE id = 1", [])?;
    tx.commit().context("Failed to commit the apply transaction")?;
    Ok(stats)
}

// ── full snapshot ──

/// What a full sync tells the standby up front.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotTable {
    pub name: String,
    pub create_sql: String,
    pub index_sql: Vec<String>,
    pub cols: Vec<String>,
    pub key: Vec<String>,
}

impl From<&TableSchema> for SnapshotTable {
    fn from(s: &TableSchema) -> Self {
        SnapshotTable {
            name: s.name.clone(),
            create_sql: s.create_sql.clone(),
            index_sql: s.index_sql.clone(),
            cols: s.cols.clone(),
            key: s.key.clone(),
        }
    }
}

/// Read up to `limit` rows of a table after `cursor`, for a full sync. Pass
/// the returned cursor back to continue; `None` means the table is done.
pub fn snapshot_rows(
    conn: &Connection,
    s: &TableSchema,
    cursor: i64,
    limit: usize,
    byte_budget: usize,
) -> Result<(Vec<Vec<Cell>>, Option<i64>)> {
    let cols = s
        .cols
        .iter()
        .map(|c| if c == "rowid" && s.keyed_by_rowid { "rowid".to_string() } else { quote_ident(c) })
        .collect::<Vec<_>>()
        .join(", ");
    let n = s.cols.len();
    let mut rows = Vec::new();
    let mut bytes = 0usize;
    if s.has_rowid {
        let sql = format!(
            "SELECT rowid, {cols} FROM {} WHERE rowid > ?1 ORDER BY rowid LIMIT ?2",
            quote_ident(&s.name)
        );
        let mut stmt = conn.prepare_cached(&sql)?;
        let mut q = stmt.query(params![cursor, limit as i64])?;
        let mut last = cursor;
        let mut count = 0usize;
        while let Some(r) = q.next()? {
            last = r.get::<_, i64>(0)?;
            let row: Vec<Cell> = (0..n)
                .map(|i| Cell::from_ref(r.get_ref(i + 1).expect("column in range")))
                .collect();
            bytes += row_bytes(&row);
            rows.push(row);
            count += 1;
            if bytes >= byte_budget {
                return Ok((rows, Some(last)));
            }
        }
        Ok((rows, if count == limit { Some(last) } else { None }))
    } else {
        // WITHOUT ROWID: page by offset in key order.
        let order = s.key.iter().map(|c| quote_ident(c)).collect::<Vec<_>>().join(", ");
        let sql = format!(
            "SELECT {cols} FROM {} ORDER BY {order} LIMIT ?1 OFFSET ?2",
            quote_ident(&s.name)
        );
        let mut stmt = conn.prepare_cached(&sql)?;
        let mut q = stmt.query(params![limit as i64, cursor])?;
        let mut count = 0usize;
        while let Some(r) = q.next()? {
            let row: Vec<Cell> = (0..n)
                .map(|i| Cell::from_ref(r.get_ref(i).expect("column in range")))
                .collect();
            bytes += row_bytes(&row);
            rows.push(row);
            count += 1;
            if bytes >= byte_budget {
                return Ok((rows, Some(cursor + count as i64)));
            }
        }
        Ok((rows, if count == limit { Some(cursor + count as i64) } else { None }))
    }
}

fn row_bytes(row: &[Cell]) -> usize {
    row.iter()
        .map(|c| match c {
            Cell::Text(t) => t.len() + 2,
            Cell::Blob(b) => b.len() + 2,
            _ => 9,
        })
        .sum()
}

/// Prepare the standby for a full sync: make sure every table exists, then
/// empty them. Runs in one transaction with the triggers muted. Marks the node
/// as needing a full sync until [`finish_full_sync`] runs.
pub fn begin_full_sync(conn: &mut Connection, tables: &[SnapshotTable]) -> Result<()> {
    with_foreign_keys_off(conn, |conn| begin_full_sync_inner(conn, tables))
}

fn begin_full_sync_inner(conn: &mut Connection, tables: &[SnapshotTable]) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute("UPDATE ha_guard SET mute = 1 WHERE id = 1", [])?;
    tx.execute(
        "UPDATE ha_node SET needs_full_sync = 1, updated_at = ?1 WHERE id = 1",
        [voltrus::timeutil::now_secs()],
    )?;
    for t in tables {
        anyhow::ensure!(is_replicated(&t.name), "table {} is not replicated", t.name);
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            [&t.name],
            |r| r.get::<_, i64>(0),
        )? != 0;
        if !exists {
            anyhow::ensure!(
                t.create_sql.trim_start().to_uppercase().starts_with("CREATE TABLE"),
                "refusing a schema statement that is not CREATE TABLE"
            );
            tx.execute_batch(&t.create_sql)?;
            for idx in &t.index_sql {
                anyhow::ensure!(
                    idx.trim_start().to_uppercase().starts_with("CREATE ")
                        && idx.to_uppercase().contains("INDEX"),
                    "refusing a schema statement that is not CREATE INDEX"
                );
                tx.execute_batch(idx)?;
            }
        }
        tx.execute(&format!("DELETE FROM {}", quote_ident(&t.name)), [])?;
    }
    tx.execute("UPDATE ha_guard SET mute = 0 WHERE id = 1", [])?;
    tx.commit()?;
    Ok(())
}

/// Record a completed full sync.
pub fn finish_full_sync(conn: &mut Connection, epoch: u64, snapshot_seq: u64) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE ha_node SET epoch = ?1, applied_seq = ?2, needs_full_sync = 0, updated_at = ?3
         WHERE id = 1",
        params![epoch as i64, snapshot_seq as i64, voltrus::timeutil::now_secs()],
    )?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(path: &std::path::Path) -> Connection {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
        conn.execute_batch(
            "CREATE TABLE t_int (id INTEGER PRIMARY KEY AUTOINCREMENT, v TEXT);
             CREATE TABLE t_text (k TEXT PRIMARY KEY, v REAL, n INTEGER);
             CREATE TABLE t_comp (a TEXT NOT NULL, b INTEGER NOT NULL, blob BLOB, PRIMARY KEY (a, b));
             CREATE TABLE t_rowid (x TEXT, y INTEGER);
             CREATE TABLE t_uniq (id INTEGER PRIMARY KEY, name TEXT UNIQUE NOT NULL);
             CREATE TABLE license_activation (id INTEGER PRIMARY KEY, license_key TEXT);
             CREATE INDEX idx_t_int_v ON t_int(v);",
        )
        .unwrap();
        init_schema(&conn).unwrap();
        install_triggers(&conn).unwrap();
        conn
    }

    /// Table contents in key order. Tables keyed by a column are compared without
    /// their implicit rowid, which is a local detail of each node.
    fn dump(conn: &Connection) -> BTreeMap<String, Vec<Vec<String>>> {
        let mut out = BTreeMap::new();
        for (name, schema) in table_schemas(conn).unwrap() {
            let cols = schema
                .cols
                .iter()
                .map(|c| if c == "rowid" { "rowid".to_string() } else { quote_ident(c) })
                .collect::<Vec<_>>();
            let order = (1..=cols.len()).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT {} FROM {} ORDER BY {order}",
                    cols.join(", "),
                    quote_ident(&name)
                ))
                .unwrap();
            let n = stmt.column_count();
            let rows = stmt
                .query_map([], |r| {
                    (0..n)
                        .map(|i| Ok(format!("{:?}", Cell::from_ref(r.get_ref(i)?))))
                        .collect::<rusqlite::Result<Vec<_>>>()
                })
                .unwrap()
                .collect::<std::result::Result<Vec<_>, _>>()
                .unwrap();
            out.insert(name, rows);
        }
        out
    }

    /// Ship everything logged on `src` after `from` to `dst`, in batches.
    fn ship(src: &Connection, dst: &mut Connection, from: u64, batch: usize) -> u64 {
        let schemas = table_schemas(src).unwrap();
        let mut pos = from;
        loop {
            let entries = read_log(src, pos, batch).unwrap();
            if entries.is_empty() {
                return pos;
            }
            let b = build_batch(src, &schemas, &entries).unwrap();
            // Through the wire format, like the real link.
            let bytes = rmp_serde::to_vec_named(&b).unwrap();
            let b: Batch = rmp_serde::from_slice(&bytes).unwrap();
            apply_changes(
                dst,
                &b.tables,
                Some(Position { epoch: 1, applied_seq: b.seq_to, needs_full_sync: false }),
            )
            .unwrap();
            pos = b.seq_to;
        }
    }

    fn pair() -> (tempfile::TempDir, Connection, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let a = open(&dir.path().join("a.db"));
        let b = open(&dir.path().join("b.db"));
        save_node(
            &b,
            &NodeRecord {
                node_id: "b".into(),
                role: StoredRole::Standby,
                epoch: 1,
                applied_seq: 0,
                needs_full_sync: false,
                peer_running: vec![],
                last_failover_at: None,
                last_failover_reason: None,
            },
        )
        .unwrap();
        (dir, a, b)
    }

    #[test]
    fn replicates_every_kind_of_key_through_the_wire_format() {
        let (_d, a, mut b) = pair();
        a.execute_batch(
            "INSERT INTO t_int (v) VALUES ('one'), ('two'), ('three');
             INSERT INTO t_text VALUES ('k1', 1.5, 7), ('k2', NULL, NULL), ('it''s \"quoted\"', 2.0, 1);
             INSERT INTO t_comp VALUES ('a', 1, x'00ff10'), ('a', 2, NULL), ('b', 1, x'');
             INSERT INTO t_rowid VALUES ('r1', 1), ('r2', 2), ('r2', 2);
             INSERT INTO t_uniq VALUES (1, 'x'), (2, 'y');",
        )
        .unwrap();
        let mut pos = ship(&a, &mut b, 0, 4);
        assert_eq!(dump(&a), dump(&b));

        a.execute_batch(
            "UPDATE t_int SET v = 'uno' WHERE id = 1;
             UPDATE t_text SET k = 'renamed' WHERE k = 'k1';
             UPDATE t_comp SET b = 9 WHERE a = 'a' AND b = 1;
             DELETE FROM t_int WHERE id = 2;
             DELETE FROM t_comp WHERE a = 'b';
             DELETE FROM t_rowid WHERE x = 'r2';
             UPDATE t_rowid SET y = 10 WHERE x = 'r1';
             UPDATE t_uniq SET name = 'z' WHERE id = 1;
             INSERT OR REPLACE INTO t_uniq VALUES (5, 'y');",
        )
        .unwrap();
        pos = ship(&a, &mut b, pos, 3);
        assert_eq!(dump(&a), dump(&b));
        assert_eq!(
            b.query_row("SELECT applied_seq FROM ha_node", [], |r| r.get::<_, i64>(0)).unwrap(),
            pos as i64
        );
    }

    #[test]
    fn randomised_changes_converge() {
        let (_d, a, mut b) = pair();
        let mut x: u64 = 0x2545F4914F6CDD1D;
        let mut next = |m: u64| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x % m
        };
        let mut pos = 0;
        for round in 0..40 {
            for _ in 0..50 {
                let k = next(12) as i64;
                match next(7) {
                    0 | 1 => {
                        a.execute("INSERT INTO t_int (v) VALUES (?1)", [format!("v{}", next(100))])
                            .unwrap();
                    }
                    2 => {
                        a.execute(
                            "INSERT OR REPLACE INTO t_text VALUES (?1, ?2, ?3)",
                            params![format!("k{k}"), next(1000) as f64 / 8.0, k],
                        )
                        .unwrap();
                    }
                    3 => {
                        a.execute(
                            "INSERT OR REPLACE INTO t_comp VALUES (?1, ?2, ?3)",
                            params![format!("a{}", k % 3), k, vec![k as u8; (k % 4) as usize]],
                        )
                        .unwrap();
                    }
                    4 => {
                        a.execute("DELETE FROM t_int WHERE id % 5 = ?1", [k % 5]).unwrap();
                    }
                    5 => {
                        a.execute("INSERT INTO t_rowid VALUES (?1, ?2)", params![format!("x{k}"), k])
                            .unwrap();
                    }
                    _ => {
                        a.execute("DELETE FROM t_rowid WHERE y = ?1", [k]).unwrap();
                        a.execute("DELETE FROM t_text WHERE n = ?1", [k]).unwrap();
                    }
                }
            }
            if round % 3 == 0 {
                pos = ship(&a, &mut b, pos, 37);
                assert_eq!(dump(&a), dump(&b), "diverged in round {round}");
            }
        }
        ship(&a, &mut b, pos, 100);
        assert_eq!(dump(&a), dump(&b));
    }

    #[test]
    fn the_standbys_own_triggers_stay_silent_while_applying() {
        let (_d, a, mut b) = pair();
        a.execute("INSERT INTO t_int (v) VALUES ('x')", []).unwrap();
        ship(&a, &mut b, 0, 10);
        assert_eq!(max_seq(&b).unwrap(), 0, "applying must not log on the standby");
        // Outside an apply the standby's triggers work again.
        b.execute("INSERT INTO t_int (v) VALUES ('local')", []).unwrap();
        assert_eq!(max_seq(&b).unwrap(), 1);
        assert_eq!(
            b.query_row("SELECT mute FROM ha_guard", [], |r| r.get::<_, i64>(0)).unwrap(),
            0
        );
    }

    #[test]
    fn a_failed_apply_rolls_back_and_unmutes() {
        let (_d, _a, mut b) = pair();
        let bad = TableChanges {
            name: "t_int".into(),
            cols: vec!["id".into(), "no_such_column".into()],
            key: vec!["id".into()],
            rows: vec![vec![Cell::Int(1), Cell::Text("x".into())]],
            ..Default::default()
        };
        assert!(apply_changes(&mut b, &[bad], None).is_err());
        b.execute("INSERT INTO t_int (v) VALUES ('after')", []).unwrap();
        assert_eq!(max_seq(&b).unwrap(), 1, "the guard is released after a failed apply");

        let injected = TableChanges {
            name: "t_int\"; DROP TABLE users; --".into(),
            ..Default::default()
        };
        assert!(apply_changes(&mut b, &[injected], None).is_err());
        let node = TableChanges { name: "ha_node".into(), ..Default::default() };
        assert!(apply_changes(&mut b, &[node], None).is_err(), "bookkeeping tables never replicate");
        let lic = TableChanges { name: "license_activation".into(), ..Default::default() };
        assert!(apply_changes(&mut b, &[lic], None).is_err(), "the licence never replicates");
    }

    #[test]
    fn repeated_changes_collapse_and_bulk_deletes_become_runs() {
        let (_d, a, _b) = pair();
        a.execute_batch("INSERT INTO t_int (v) VALUES ('a'); UPDATE t_int SET v = 'b'; UPDATE t_int SET v = 'c';")
            .unwrap();
        let schemas = table_schemas(&a).unwrap();
        let batch = build_batch(&a, &schemas, &read_log(&a, 0, 100).unwrap()).unwrap();
        assert_eq!(batch.tables.len(), 1);
        assert_eq!(batch.tables[0].rows.len(), 1, "three changes to one row ship one image");
        assert_eq!(batch.tables[0].rows[0][1], Cell::Text("c".into()));

        let mark = max_seq(&a).unwrap();
        a.execute_batch("WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 5000) INSERT INTO t_int (v) SELECT 'bulk' FROM n;")
            .unwrap();
        let mark2 = max_seq(&a).unwrap();
        a.execute("DELETE FROM t_int WHERE id > 1", []).unwrap();
        let entries = read_log(&a, mark2, 100_000).unwrap();
        assert!(entries.len() >= 5000);
        let batch = build_batch(&a, &schemas, &entries).unwrap();
        let t = &batch.tables[0];
        assert!(t.rows.is_empty() && t.deletes.is_empty());
        assert_eq!(t.delete_runs, vec![(2, 5001)], "5000 deleted rows are one run");
        assert_eq!(batch.change_count(), 5000);
        let _ = mark;
    }

    #[test]
    fn a_row_deleted_after_it_was_logged_ships_as_a_delete() {
        let (_d, a, mut b) = pair();
        a.execute_batch("INSERT INTO t_text VALUES ('gone', 1.0, 1); INSERT INTO t_text VALUES ('stays', 2.0, 2);")
            .unwrap();
        let first = max_seq(&a).unwrap();
        a.execute("DELETE FROM t_text WHERE k = 'gone'", []).unwrap();
        // Ship only the insert entries: the image fetch finds the row already deleted.
        let schemas = table_schemas(&a).unwrap();
        let entries = read_log(&a, 0, 2).unwrap();
        assert!(max_seq(&a).unwrap() > first);
        let batch = build_batch(&a, &schemas, &entries).unwrap();
        apply_changes(&mut b, &batch.tables, None).unwrap();
        let n: i64 = b.query_row("SELECT COUNT(*) FROM t_text", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
        ship(&a, &mut b, batch.seq_to, 10);
        assert_eq!(dump(&a), dump(&b));
    }

    #[test]
    fn full_sync_creates_missing_tables_and_replaces_local_rows() {
        let (dir, a, mut b) = pair();
        a.execute_batch(
            "CREATE TABLE only_on_a (id INTEGER PRIMARY KEY, v TEXT);
             CREATE INDEX idx_only ON only_on_a(v);",
        )
        .unwrap();
        install_triggers(&a).unwrap();
        a.execute_batch(
            "INSERT INTO only_on_a VALUES (1, 'a'), (2, 'b');
             INSERT INTO t_int (v) VALUES ('from-a');
             INSERT INTO t_rowid VALUES ('r', 1);
             INSERT INTO license_activation VALUES (1, 'node-a-key');",
        )
        .unwrap();
        b.execute_batch("INSERT INTO t_int (v) VALUES ('stale-local'); INSERT INTO license_activation VALUES (1, 'node-b-key');")
            .unwrap();

        let schemas = table_schemas(&a).unwrap();
        let tables: Vec<SnapshotTable> = schemas.values().map(SnapshotTable::from).collect();
        a.execute_batch("PRAGMA wal_checkpoint(PASSIVE)").unwrap();
        let reader = Connection::open_with_flags(
            dir.path().join("a.db"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        reader.execute_batch("BEGIN").unwrap();
        let snap_seq = max_seq(&reader).unwrap();

        let local_log = max_seq(&b).unwrap();
        begin_full_sync(&mut b, &tables).unwrap();
        assert!(load_node(&b).unwrap().unwrap().needs_full_sync);
        for t in &tables {
            let s = &schemas[&t.name];
            let mut cursor = 0;
            loop {
                let (rows, next) = snapshot_rows(&reader, s, cursor, 2, 1 << 20).unwrap();
                apply_changes(
                    &mut b,
                    &[TableChanges {
                        name: t.name.clone(),
                        cols: t.cols.clone(),
                        key: t.key.clone(),
                        rows,
                        ..Default::default()
                    }],
                    None,
                )
                .unwrap();
                match next {
                    Some(c) => cursor = c,
                    None => break,
                }
            }
        }
        finish_full_sync(&mut b, 3, snap_seq).unwrap();

        assert_eq!(dump(&a), dump(&b));
        let node = load_node(&b).unwrap().unwrap();
        assert_eq!((node.epoch, node.applied_seq, node.needs_full_sync), (3, snap_seq, false));
        let lic: String = b
            .query_row("SELECT license_key FROM license_activation", [], |r| r.get(0))
            .unwrap();
        assert_eq!(lic, "node-b-key", "the machine-bound licence is never replicated");
        assert_eq!(max_seq(&b).unwrap(), local_log, "a full sync logs nothing on the standby");
    }

    #[test]
    fn log_floor_and_position_rules() {
        let (_d, a, _b) = pair();
        assert_eq!(max_seq(&a).unwrap(), 0);
        assert_eq!(log_floor(&a).unwrap(), 0);
        a.execute_batch("INSERT INTO t_int (v) VALUES ('1'), ('2'), ('3'), ('4');").unwrap();
        assert_eq!((log_floor(&a).unwrap(), max_seq(&a).unwrap()), (0, 4));
        prune_log(&a, 2).unwrap();
        assert_eq!((log_floor(&a).unwrap(), max_seq(&a).unwrap()), (2, 4));
        prune_log(&a, 4).unwrap();
        assert_eq!(
            (log_floor(&a).unwrap(), max_seq(&a).unwrap()),
            (4, 4),
            "an emptied log keeps counting where it stopped"
        );
        a.execute_batch("INSERT INTO t_int (v) VALUES ('5');").unwrap();
        assert_eq!((log_floor(&a).unwrap(), max_seq(&a).unwrap()), (4, 5));
        for _ in 0..10 {
            a.execute_batch("INSERT INTO t_int (v) VALUES ('x');").unwrap();
        }
        assert_eq!(trim_log(&a, 4).unwrap(), 7, "trim keeps the newest 4 of the 11 logged");
        assert_eq!(log_floor(&a).unwrap(), 11);
    }

    #[test]
    fn tables_created_later_are_picked_up_and_teardown_cleans_up() {
        let (_d, a, _b) = pair();
        assert_eq!(install_triggers(&a).unwrap(), 0, "second call finds nothing to do");
        a.execute_batch("CREATE TABLE history_blocks (id INTEGER PRIMARY KEY, packed BLOB);").unwrap();
        assert_eq!(install_triggers(&a).unwrap(), 3);
        a.execute("INSERT INTO history_blocks VALUES (1, x'0102')", []).unwrap();
        let e = read_log(&a, 0, 10).unwrap();
        assert_eq!((e[0].tbl.as_str(), e[0].pk.as_str()), ("history_blocks", "[1]"));
        teardown(&a).unwrap();
        assert!(!has_triggers(&a).unwrap());
        a.execute("INSERT INTO history_blocks VALUES (2, x'03')", []).unwrap();
    }

    #[test]
    fn node_record_round_trips() {
        let (_d, a, _b) = pair();
        assert_eq!(load_node(&a).unwrap(), None);
        let n = NodeRecord {
            node_id: "node-a".into(),
            role: StoredRole::Primary,
            epoch: 7,
            applied_seq: 99,
            needs_full_sync: true,
            peer_running: vec!["plc_1".into(), "s7_2".into()],
            last_failover_at: Some(1_700_000_000),
            last_failover_reason: Some("test".into()),
        };
        save_node(&a, &n).unwrap();
        assert_eq!(load_node(&a).unwrap(), Some(n.clone()));
        save_node(&a, &NodeRecord { epoch: 8, ..n.clone() }).unwrap();
        assert_eq!(load_node(&a).unwrap().unwrap().epoch, 8);
    }

    /// The core leaves foreign keys enforced on its connection. A cascaded delete
    /// must replicate as deletes of the children, and applying rows in any order
    /// must not trip the constraint on the standby.
    #[test]
    fn foreign_keys_enforced_on_both_nodes_do_not_break_replication() {
        let (_d, a, mut b) = pair();
        for c in [&a, &b] {
            c.execute_batch(
                "CREATE TABLE parent (id INTEGER PRIMARY KEY, name TEXT);
                 CREATE TABLE child (id INTEGER PRIMARY KEY, parent_id INTEGER NOT NULL REFERENCES parent(id) ON DELETE CASCADE, v TEXT);
                 PRAGMA foreign_keys = ON;",
            )
            .unwrap();
            install_triggers(c).unwrap();
        }
        a.execute_batch(
            "INSERT INTO parent VALUES (1, 'p1'), (2, 'p2');
             INSERT INTO child VALUES (10, 1, 'a'), (11, 1, 'b'), (12, 2, 'c');",
        )
        .unwrap();
        // Children are logged before the parents of other keys can be: a batch of two
        // entries holds only the children.
        let schemas = table_schemas(&a).unwrap();
        let child_only = vec![LogEntry { seq: 1, tbl: "child".into(), op: OP_UPSERT, pk: "[10]".into() }];
        let batch = build_batch(&a, &schemas, &child_only).unwrap();
        apply_changes(&mut b, &batch.tables, None).expect("a child may arrive before its parent");
        b.execute_batch("DELETE FROM child").unwrap();
        ship(&a, &mut b, 0, 100);
        assert_eq!(dump(&a), dump(&b));

        let before = max_seq(&a).unwrap();
        a.execute("DELETE FROM parent WHERE id = 1", []).unwrap();
        let logged: Vec<String> = read_log(&a, before, 100)
            .unwrap()
            .into_iter()
            .map(|e| format!("{}:{}:{}", e.tbl, e.op, e.pk))
            .collect();
        assert!(
            logged.contains(&"child:2:[10]".to_string()) && logged.contains(&"child:2:[11]".to_string()),
            "cascaded deletes are logged: {logged:?}"
        );
        ship(&a, &mut b, before, 100);
        assert_eq!(dump(&a), dump(&b));
        let fk: i64 = b.query_row("PRAGMA foreign_keys", [], |r| r.get(0)).unwrap();
        assert_eq!(fk, 1, "enforcement is restored after an apply");
    }
}
