//! The energy tables. Created by the add-on at start (like the Part 11
//! tables), before hot standby installs its replication triggers.

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use super::counter::Quality;
use super::tariff::{AlertConfig, Band, Golongan, Site, Tariff, TariffValues, WbpWindow};

const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS energy_site (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        timezone TEXT NOT NULL DEFAULT 'Asia/Jakarta',
        wbp_windows TEXT NOT NULL,
        contracted_kva REAL,
        alerts TEXT NOT NULL DEFAULT '{}',
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS energy_tariff (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        site_id INTEGER NOT NULL,
        valid_from TEXT NOT NULL,
        golongan TEXT NOT NULL CHECK (golongan IN ('I-3', 'I-4')),
        k_factor REAL,
        rate_lwbp REAL NOT NULL,
        rate_wbp REAL,
        rate_kvarh REAL NOT NULL,
        pf_limit REAL NOT NULL DEFAULT 0.85,
        kvarh_factor REAL,
        note TEXT NOT NULL DEFAULT '',
        created_at INTEGER NOT NULL,
        UNIQUE (site_id, valid_from)
    );
    CREATE TABLE IF NOT EXISTS energy_meter (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        site_id INTEGER NOT NULL,
        name TEXT NOT NULL,
        device_id TEXT NOT NULL,
        kwh_metric TEXT NOT NULL,
        kvarh_metric TEXT,
        kvar_metric TEXT,
        role TEXT NOT NULL CHECK (role IN ('pln_incomer', 'sub')),
        multiplier REAL NOT NULL DEFAULT 1,
        rollover REAL,
        max_gap_secs INTEGER NOT NULL DEFAULT 900,
        enabled INTEGER NOT NULL DEFAULT 1,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_energy_meter_site ON energy_meter(site_id);
    CREATE TABLE IF NOT EXISTS energy_hour (
        meter_id INTEGER NOT NULL,
        hour_start INTEGER NOT NULL,
        kwh REAL NOT NULL,
        kvarh REAL,
        band TEXT NOT NULL CHECK (band IN ('LWBP', 'WBP')),
        quality TEXT NOT NULL CHECK (quality IN ('ok', 'gap', 'estimated')),
        computed_at INTEGER NOT NULL,
        PRIMARY KEY (meter_id, hour_start)
    );
    CREATE TABLE IF NOT EXISTS energy_bill (
        site_id INTEGER NOT NULL,
        month TEXT NOT NULL,
        pln TEXT NOT NULL,
        computed TEXT NOT NULL,
        entered_by TEXT NOT NULL,
        entered_at INTEGER NOT NULL,
        PRIMARY KEY (site_id, month)
    );
    CREATE TABLE IF NOT EXISTS energy_notice (
        site_id INTEGER NOT NULL,
        kind TEXT NOT NULL,
        day TEXT NOT NULL,
        sent_at INTEGER NOT NULL,
        PRIMARY KEY (site_id, kind, day)
    );";

pub(crate) fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)
}

// ── Sites ──

fn site_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Site> {
    let windows: String = r.get(3)?;
    let alerts: String = r.get(5)?;
    Ok(Site {
        id: r.get(0)?,
        name: r.get(1)?,
        timezone: r.get(2)?,
        wbp_windows: serde_json::from_str(&windows)
            .unwrap_or_else(|_| WbpWindow::default_windows()),
        contracted_kva: r.get(4)?,
        alerts: serde_json::from_str(&alerts).unwrap_or_default(),
        created_at: r.get(6)?,
        updated_at: r.get(7)?,
    })
}

const SITE_COLS: &str =
    "id, name, timezone, wbp_windows, contracted_kva, alerts, created_at, updated_at";

pub fn list_sites(conn: &Connection) -> Result<Vec<Site>> {
    let mut stmt = conn.prepare(&format!("SELECT {SITE_COLS} FROM energy_site ORDER BY id"))?;
    let rows = stmt
        .query_map([], site_from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_site(conn: &Connection, id: i64) -> Result<Option<Site>> {
    Ok(conn
        .query_row(
            &format!("SELECT {SITE_COLS} FROM energy_site WHERE id = ?1"),
            [id],
            site_from_row,
        )
        .optional()?)
}

pub struct SiteFields<'a> {
    pub name: &'a str,
    pub timezone: &'a str,
    pub wbp_windows: &'a [WbpWindow],
    pub contracted_kva: Option<f64>,
    pub alerts: &'a AlertConfig,
}

pub fn insert_site(conn: &Connection, f: &SiteFields<'_>, now: i64) -> Result<i64> {
    conn.execute(
        "INSERT INTO energy_site (name, timezone, wbp_windows, contracted_kva, alerts, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![
            f.name,
            f.timezone,
            serde_json::to_string(f.wbp_windows)?,
            f.contracted_kva,
            serde_json::to_string(f.alerts)?,
            now
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_site(conn: &Connection, id: i64, f: &SiteFields<'_>, now: i64) -> Result<bool> {
    let n = conn.execute(
        "UPDATE energy_site SET name = ?1, timezone = ?2, wbp_windows = ?3, contracted_kva = ?4,
             alerts = ?5, updated_at = ?6 WHERE id = ?7",
        params![
            f.name,
            f.timezone,
            serde_json::to_string(f.wbp_windows)?,
            f.contracted_kva,
            serde_json::to_string(f.alerts)?,
            now,
            id
        ],
    )?;
    Ok(n > 0)
}

/// Delete a site with its tariffs, meters, hours, bills and notices.
pub fn delete_site(conn: &Connection, id: i64) -> Result<bool> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM energy_hour WHERE meter_id IN (SELECT id FROM energy_meter WHERE site_id = ?1)",
        [id],
    )?;
    for table in [
        "energy_meter",
        "energy_tariff",
        "energy_bill",
        "energy_notice",
    ] {
        tx.execute(&format!("DELETE FROM {table} WHERE site_id = ?1"), [id])?;
    }
    let n = tx.execute("DELETE FROM energy_site WHERE id = ?1", [id])?;
    tx.commit()?;
    Ok(n > 0)
}

// ── Tariffs ──

fn tariff_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Tariff> {
    let golongan: String = r.get(3)?;
    Ok(Tariff {
        id: r.get(0)?,
        site_id: r.get(1)?,
        valid_from: r.get(2)?,
        values: TariffValues {
            golongan: Golongan::parse(&golongan).unwrap_or(Golongan::I3),
            k_factor: r.get(4)?,
            rate_lwbp: r.get(5)?,
            rate_wbp: r.get(6)?,
            rate_kvarh: r.get(7)?,
            pf_limit: r.get(8)?,
            kvarh_factor: r.get(9)?,
        },
        note: r.get(10)?,
    })
}

pub fn list_tariffs(conn: &Connection, site_id: i64) -> Result<Vec<Tariff>> {
    let mut stmt = conn.prepare(
        "SELECT id, site_id, valid_from, golongan, k_factor, rate_lwbp, rate_wbp, rate_kvarh,
                pf_limit, kvarh_factor, note
         FROM energy_tariff WHERE site_id = ?1 ORDER BY valid_from",
    )?;
    let rows = stmt
        .query_map([site_id], tariff_from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_tariff(conn: &Connection, id: i64) -> Result<Option<Tariff>> {
    Ok(conn
        .query_row(
            "SELECT id, site_id, valid_from, golongan, k_factor, rate_lwbp, rate_wbp, rate_kvarh,
                    pf_limit, kvarh_factor, note
             FROM energy_tariff WHERE id = ?1",
            [id],
            tariff_from_row,
        )
        .optional()?)
}

/// Insert or replace the tariff of `site_id` starting in `valid_from`.
pub fn upsert_tariff(
    conn: &Connection,
    site_id: i64,
    valid_from: &str,
    v: &TariffValues,
    note: &str,
    now: i64,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO energy_tariff (site_id, valid_from, golongan, k_factor, rate_lwbp, rate_wbp,
             rate_kvarh, pf_limit, kvarh_factor, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT (site_id, valid_from) DO UPDATE SET golongan = excluded.golongan,
             k_factor = excluded.k_factor, rate_lwbp = excluded.rate_lwbp,
             rate_wbp = excluded.rate_wbp, rate_kvarh = excluded.rate_kvarh,
             pf_limit = excluded.pf_limit, kvarh_factor = excluded.kvarh_factor,
             note = excluded.note",
        params![
            site_id,
            valid_from,
            v.golongan.as_str(),
            v.k_factor,
            v.rate_lwbp,
            v.rate_wbp,
            v.rate_kvarh,
            v.pf_limit,
            v.kvarh_factor,
            note,
            now
        ],
    )?;
    Ok(conn.query_row(
        "SELECT id FROM energy_tariff WHERE site_id = ?1 AND valid_from = ?2",
        params![site_id, valid_from],
        |r| r.get(0),
    )?)
}

pub fn update_tariff(
    conn: &Connection,
    id: i64,
    valid_from: &str,
    v: &TariffValues,
    note: &str,
) -> Result<bool> {
    let n = conn.execute(
        "UPDATE energy_tariff SET valid_from = ?1, golongan = ?2, k_factor = ?3, rate_lwbp = ?4,
             rate_wbp = ?5, rate_kvarh = ?6, pf_limit = ?7, kvarh_factor = ?8, note = ?9
         WHERE id = ?10",
        params![
            valid_from,
            v.golongan.as_str(),
            v.k_factor,
            v.rate_lwbp,
            v.rate_wbp,
            v.rate_kvarh,
            v.pf_limit,
            v.kvarh_factor,
            note,
            id
        ],
    )?;
    Ok(n > 0)
}

pub fn delete_tariff(conn: &Connection, id: i64) -> Result<bool> {
    Ok(conn.execute("DELETE FROM energy_tariff WHERE id = ?1", [id])? > 0)
}

// ── Meters ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeterRole {
    /// PLN's supply point: billed.
    PlnIncomer,
    /// A sub-meter: computed hourly, not billed.
    Sub,
}

impl MeterRole {
    pub fn as_str(self) -> &'static str {
        match self {
            MeterRole::PlnIncomer => "pln_incomer",
            MeterRole::Sub => "sub",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meter {
    pub id: i64,
    pub site_id: i64,
    pub name: String,
    /// Historian device id and tag (metric) names, as in `/api/v1/history`.
    pub device_id: String,
    /// Cumulative active energy counter.
    pub kwh_metric: String,
    /// Cumulative reactive energy counter.
    pub kvarh_metric: Option<String>,
    /// Reactive power, integrated over time when there is no kVArh counter.
    pub kvar_metric: Option<String>,
    pub role: MeterRole,
    /// CT ratio × PT ratio × unit scale (0.001 for Wh/varh counters).
    pub multiplier: f64,
    /// Raw value the counter wraps at, if it can wrap (4294967296 for 32 bits).
    pub rollover: Option<f64>,
    /// Longest time between samples that still counts as continuous data.
    pub max_gap_secs: i64,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

const METER_COLS: &str =
    "id, site_id, name, device_id, kwh_metric, kvarh_metric, kvar_metric, role,
    multiplier, rollover, max_gap_secs, enabled, created_at, updated_at";

fn meter_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Meter> {
    let role: String = r.get(7)?;
    Ok(Meter {
        id: r.get(0)?,
        site_id: r.get(1)?,
        name: r.get(2)?,
        device_id: r.get(3)?,
        kwh_metric: r.get(4)?,
        kvarh_metric: r.get(5)?,
        kvar_metric: r.get(6)?,
        role: if role == "pln_incomer" {
            MeterRole::PlnIncomer
        } else {
            MeterRole::Sub
        },
        multiplier: r.get(8)?,
        rollover: r.get(9)?,
        max_gap_secs: r.get(10)?,
        enabled: r.get::<_, i64>(11)? != 0,
        created_at: r.get(12)?,
        updated_at: r.get(13)?,
    })
}

pub fn list_meters(conn: &Connection, site_id: Option<i64>) -> Result<Vec<Meter>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {METER_COLS} FROM energy_meter WHERE ?1 IS NULL OR site_id = ?1 ORDER BY id"
    ))?;
    let rows = stmt
        .query_map([site_id], meter_from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_meter(conn: &Connection, id: i64) -> Result<Option<Meter>> {
    Ok(conn
        .query_row(
            &format!("SELECT {METER_COLS} FROM energy_meter WHERE id = ?1"),
            [id],
            meter_from_row,
        )
        .optional()?)
}

pub fn insert_meter(conn: &Connection, m: &Meter, now: i64) -> Result<i64> {
    conn.execute(
        "INSERT INTO energy_meter (site_id, name, device_id, kwh_metric, kvarh_metric, kvar_metric,
             role, multiplier, rollover, max_gap_secs, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
        params![
            m.site_id,
            m.name,
            m.device_id,
            m.kwh_metric,
            m.kvarh_metric,
            m.kvar_metric,
            m.role.as_str(),
            m.multiplier,
            m.rollover,
            m.max_gap_secs,
            m.enabled as i64,
            now
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_meter(conn: &Connection, m: &Meter, now: i64) -> Result<bool> {
    let n = conn.execute(
        "UPDATE energy_meter SET name = ?1, device_id = ?2, kwh_metric = ?3, kvarh_metric = ?4,
             kvar_metric = ?5, role = ?6, multiplier = ?7, rollover = ?8, max_gap_secs = ?9,
             enabled = ?10, updated_at = ?11
         WHERE id = ?12",
        params![
            m.name,
            m.device_id,
            m.kwh_metric,
            m.kvarh_metric,
            m.kvar_metric,
            m.role.as_str(),
            m.multiplier,
            m.rollover,
            m.max_gap_secs,
            m.enabled as i64,
            now,
            m.id
        ],
    )?;
    Ok(n > 0)
}

pub fn delete_meter(conn: &Connection, id: i64) -> Result<bool> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM energy_hour WHERE meter_id = ?1", [id])?;
    let n = tx.execute("DELETE FROM energy_meter WHERE id = ?1", [id])?;
    tx.commit()?;
    Ok(n > 0)
}

// ── Hours ──

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HourRow {
    pub meter_id: i64,
    pub hour_start: i64,
    pub kwh: f64,
    pub kvarh: Option<f64>,
    pub band: Band,
    pub quality: Quality,
}

/// Replace the rows of `meter_id` for hours in `[from, to)` with `rows`.
pub fn replace_hours(
    conn: &Connection,
    meter_id: i64,
    from: i64,
    to: i64,
    rows: &[HourRow],
    now: i64,
) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM energy_hour WHERE meter_id = ?1 AND hour_start >= ?2 AND hour_start < ?3",
        params![meter_id, from, to],
    )?;
    {
        let mut stmt = tx.prepare_cached(
            "INSERT OR REPLACE INTO energy_hour (meter_id, hour_start, kwh, kvarh, band, quality, computed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;
        for r in rows {
            stmt.execute(params![
                meter_id,
                r.hour_start,
                r.kwh,
                r.kvarh,
                r.band.as_str(),
                r.quality.as_str(),
                now
            ])?;
        }
    }
    tx.commit().context("Commit energy hours failed")?;
    Ok(())
}

/// Rows of the given meters with `from <= hour_start < to`, by hour.
pub fn hours_for_meters(
    conn: &Connection,
    meter_ids: &[i64],
    from: i64,
    to: i64,
) -> Result<Vec<HourRow>> {
    if meter_ids.is_empty() {
        return Ok(Vec::new());
    }
    let ids = meter_ids
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let mut stmt = conn.prepare(&format!(
        "SELECT meter_id, hour_start, kwh, kvarh, band, quality FROM energy_hour
         WHERE meter_id IN ({ids}) AND hour_start >= ?1 AND hour_start < ?2
         ORDER BY hour_start, meter_id"
    ))?;
    let rows = stmt
        .query_map(params![from, to], |r| {
            let band: String = r.get(4)?;
            let quality: String = r.get(5)?;
            Ok(HourRow {
                meter_id: r.get(0)?,
                hour_start: r.get(1)?,
                kwh: r.get(2)?,
                kvarh: r.get(3)?,
                band: Band::parse(&band),
                quality: Quality::parse(&quality),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Latest computed hour of a meter.
pub fn last_hour(conn: &Connection, meter_id: i64) -> Result<Option<i64>> {
    Ok(conn.query_row(
        "SELECT MAX(hour_start) FROM energy_hour WHERE meter_id = ?1",
        [meter_id],
        |r| r.get(0),
    )?)
}

/// Earliest `gap` hour of a meter at or after `since`.
pub fn first_gap_since(conn: &Connection, meter_id: i64, since: i64) -> Result<Option<i64>> {
    Ok(conn.query_row(
        "SELECT MIN(hour_start) FROM energy_hour WHERE meter_id = ?1 AND hour_start >= ?2 AND quality = 'gap'",
        params![meter_id, since],
        |r| r.get(0),
    )?)
}

/// Set the band of every stored hour of a site again (after its WBP windows or
/// timezone changed).
pub fn reband_site(conn: &Connection, site: &Site) -> Result<usize> {
    let clock = site.clock();
    let tx = conn.unchecked_transaction()?;
    let hours: Vec<(i64, i64)> = {
        let mut stmt = tx.prepare(
            "SELECT h.meter_id, h.hour_start FROM energy_hour h
             JOIN energy_meter m ON m.id = h.meter_id WHERE m.site_id = ?1",
        )?;
        stmt.query_map([site.id], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?
    };
    {
        let mut stmt = tx.prepare_cached(
            "UPDATE energy_hour SET band = ?1 WHERE meter_id = ?2 AND hour_start = ?3",
        )?;
        for (meter, hour) in &hours {
            stmt.execute(params![
                clock.band(*hour, &site.wbp_windows).as_str(),
                meter,
                hour
            ])?;
        }
    }
    tx.commit()?;
    Ok(hours.len())
}

// ── PLN bills (reconciliation) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredBill {
    pub site_id: i64,
    pub month: String,
    pub pln: serde_json::Value,
    /// The shadow bill when PLN's figures were entered.
    pub computed: serde_json::Value,
    pub entered_by: String,
    pub entered_at: i64,
}

pub fn upsert_bill(conn: &Connection, b: &StoredBill) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO energy_bill (site_id, month, pln, computed, entered_by, entered_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            b.site_id,
            b.month,
            b.pln.to_string(),
            b.computed.to_string(),
            b.entered_by,
            b.entered_at
        ],
    )?;
    Ok(())
}

pub fn list_bills(conn: &Connection, site_id: i64) -> Result<Vec<StoredBill>> {
    let mut stmt = conn.prepare(
        "SELECT site_id, month, pln, computed, entered_by, entered_at FROM energy_bill
         WHERE site_id = ?1 ORDER BY month DESC",
    )?;
    let rows = stmt
        .query_map([site_id], |r| {
            let pln: String = r.get(2)?;
            let computed: String = r.get(3)?;
            Ok(StoredBill {
                site_id: r.get(0)?,
                month: r.get(1)?,
                pln: serde_json::from_str(&pln).unwrap_or_default(),
                computed: serde_json::from_str(&computed).unwrap_or_default(),
                entered_by: r.get(4)?,
                entered_at: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn delete_bill(conn: &Connection, site_id: i64, month: &str) -> Result<bool> {
    Ok(conn.execute(
        "DELETE FROM energy_bill WHERE site_id = ?1 AND month = ?2",
        params![site_id, month],
    )? > 0)
}

// ── Sent notices (daily summary) ──

/// Record that `kind` was sent for `day`; false when it already was.
pub fn mark_notice(
    conn: &Connection,
    site_id: i64,
    kind: &str,
    day: &str,
    now: i64,
) -> Result<bool> {
    Ok(conn.execute(
        "INSERT OR IGNORE INTO energy_notice (site_id, kind, day, sent_at) VALUES (?1, ?2, ?3, ?4)",
        params![site_id, kind, day, now],
    )? > 0)
}

pub fn notice_sent(conn: &Connection, site_id: i64, kind: &str, day: &str) -> Result<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM energy_notice WHERE site_id = ?1 AND kind = ?2 AND day = ?3",
            params![site_id, kind, day],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}
