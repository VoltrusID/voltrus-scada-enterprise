//! REST API of Voltrus Energy, under `/api/v1/energy`.
//!
//! Reads are open to every signed-in user; configuration, PLN bill entry and
//! recomputation use the core's configuration group (admin, demo, integration).

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};
use voltrus::api::{AppError, AppState, AuthUser};
use voltrus::storage::Storage;

use super::bill::PlnFigures;
use super::job;
use super::report::{self, MonthReport};
use super::store::{self, Meter, MeterRole, SiteFields, StoredBill};
use super::tariff::{
    AlertConfig, DEFAULT_TIMEZONE, Golongan, LocalClock, Site, Tariff, TariffValues, WbpWindow,
    parse_month, tariff_for, validate_windows,
};

type ApiResult = Result<Json<Value>, AppError>;

fn check_licence(state: &AppState) -> Result<(), AppError> {
    if super::licensed(state.tier()) {
        Ok(())
    } else {
        Err(AppError::forbidden(
            "Voltrus Energy requires an Integrator or Enterprise licence. See https://voltrus.id/voltrus-scada",
        ))
    }
}

async fn db<T, F>(storage: &Storage, f: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce(&rusqlite::Connection) -> anyhow::Result<T> + Send + 'static,
{
    Ok(storage.with_conn(move |c| f(c)).await?)
}

fn now() -> i64 {
    voltrus::timeutil::now_secs()
}

fn audit(state: &AppState, auth: &AuthUser, action: &str, resource: String, details: String) {
    state.audit_logger.log(voltrus::security::AuditEvent {
        action: action.to_string(),
        username: auth.username.clone(),
        resource,
        details: Some(details),
    });
}

async fn load_site(storage: &Storage, id: i64) -> Result<Site, AppError> {
    db(storage, move |c| store::get_site(c, id))
        .await?
        .ok_or_else(|| AppError::not_found("Energy site not found"))
}

/// Things the user must still check against a real PLN bill.
fn unconfirmed() -> Value {
    json!([
        "excess_kvarh_formula",
        "wbp_hours",
        "k_factor",
        "meter_counters"
    ])
}

// ── Defaults ──

/// GET /api/v1/energy/defaults
pub async fn defaults(State(state): State<AppState>) -> ApiResult {
    check_licence(&state)?;
    let i3 = TariffValues::defaults(Golongan::I3);
    Ok(Json(json!({
        "tariffs": { "I-3": i3, "I-4": TariffValues::defaults(Golongan::I4) },
        "excess_factor": i3.excess_factor(),
        "timezone": DEFAULT_TIMEZONE,
        "wbp_windows": WbpWindow::default_windows(),
        "alerts": AlertConfig::default(),
        "tolerance": super::bill::RECONCILE_TOLERANCE,
        "excluded": report::EXCLUDED,
        "unconfirmed": unconfirmed(),
        "source": "PLN tariff 2025 (energy research §1.2); excess kVArh rule is common practice, to be confirmed",
    })))
}

// ── Sites ──

#[derive(Debug, Deserialize)]
pub struct TariffInput {
    pub valid_from: String,
    #[serde(flatten)]
    pub values: TariffValues,
    #[serde(default)]
    pub note: String,
}

impl TariffInput {
    fn validate(&self) -> Result<(), AppError> {
        parse_month(&self.valid_from).map_err(AppError::bad_request)?;
        self.values.validate().map_err(AppError::bad_request)?;
        if self.note.len() > 500 {
            return Err(AppError::bad_request("note is too long"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct SiteInput {
    pub name: Option<String>,
    pub timezone: Option<String>,
    pub wbp_windows: Option<Vec<WbpWindow>>,
    #[serde(default, deserialize_with = "double_option")]
    pub contracted_kva: Option<Option<f64>>,
    pub alerts: Option<AlertConfig>,
    /// First tariff of a new site (defaults: I-3 from this month).
    pub tariff: Option<TariffInput>,
}

/// Tells "absent" (keep) from `null` (clear).
fn double_option<'de, D>(de: D) -> Result<Option<Option<f64>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(Option::<f64>::deserialize(de)?))
}

struct ValidSite {
    name: String,
    timezone: String,
    windows: Vec<WbpWindow>,
    kva: Option<f64>,
    alerts: AlertConfig,
}

fn merge_site(input: &SiteInput, current: Option<&Site>) -> Result<ValidSite, AppError> {
    let name = input
        .name
        .clone()
        .or_else(|| current.map(|s| s.name.clone()))
        .unwrap_or_default()
        .trim()
        .to_string();
    if name.is_empty() || name.len() > 100 {
        return Err(AppError::bad_request(
            "name is required (at most 100 characters)",
        ));
    }
    let timezone = input
        .timezone
        .clone()
        .or_else(|| current.map(|s| s.timezone.clone()))
        .unwrap_or_else(|| DEFAULT_TIMEZONE.to_string());
    LocalClock::parse(&timezone).map_err(AppError::bad_request)?;
    let windows = input
        .wbp_windows
        .clone()
        .or_else(|| current.map(|s| s.wbp_windows.clone()))
        .unwrap_or_else(WbpWindow::default_windows);
    validate_windows(&windows).map_err(AppError::bad_request)?;
    let kva = match input.contracted_kva {
        Some(v) => v,
        None => current.and_then(|s| s.contracted_kva),
    };
    if let Some(k) = kva
        && !(k.is_finite() && k > 0.0)
    {
        return Err(AppError::bad_request("contracted_kva must be positive"));
    }
    let alerts = input
        .alerts
        .clone()
        .or_else(|| current.map(|s| s.alerts.clone()))
        .unwrap_or_default();
    alerts.validate().map_err(AppError::bad_request)?;
    Ok(ValidSite {
        name,
        timezone,
        windows,
        kva,
        alerts,
    })
}

/// Whether a reconciled month of the site agreed with PLN within 2%.
async fn site_confirmed(storage: &Storage, site: &Site) -> Result<bool, AppError> {
    let site_id = site.id;
    let bills = db(storage, move |c| store::list_bills(c, site_id)).await?;
    for b in bills {
        let report = month_report(storage, site, &b.month).await?;
        if report
            .reconciliation
            .as_ref()
            .is_some_and(|r| r.within_tolerance)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn site_keys(id: i64) -> Value {
    json!({
        "pf": job::pf_key(id),
        "wbp_kw": job::wbp_kw_key(id),
        "daily": job::daily_key(id),
    })
}

async fn site_detail(storage: &Storage, site: Site) -> ApiResult {
    let id = site.id;
    let (tariffs, meters) = db(storage, move |c| {
        Ok((
            store::list_tariffs(c, id)?,
            store::list_meters(c, Some(id))?,
        ))
    })
    .await?;
    let confirmed = site_confirmed(storage, &site).await?;
    let month = site.clock().month_of(now());
    Ok(Json(json!({
        "site": site,
        "current_tariff": tariff_for(&tariffs, &month),
        "tariffs": tariffs,
        "meters": meters,
        "confirmed": confirmed,
        "notification_keys": site_keys(id),
    })))
}

/// GET /api/v1/energy/sites
pub async fn list_sites(State(state): State<AppState>) -> ApiResult {
    check_licence(&state)?;
    let (sites, tariffs, meters) = db(&state.storage, |c| {
        let sites = store::list_sites(c)?;
        let mut tariffs = Vec::new();
        for s in &sites {
            tariffs.push(store::list_tariffs(c, s.id)?);
        }
        Ok((sites, tariffs, store::list_meters(c, None)?))
    })
    .await?;
    let mut out = Vec::new();
    for (site, tariffs) in sites.into_iter().zip(tariffs) {
        let month = site.clock().month_of(now());
        let confirmed = site_confirmed(&state.storage, &site).await?;
        out.push(json!({
            "site": site,
            "current_tariff": tariff_for(&tariffs, &month),
            "meter_count": meters.iter().filter(|m| m.site_id == site.id).count(),
            "confirmed": confirmed,
        }));
    }
    Ok(Json(json!(out)))
}

/// GET /api/v1/energy/sites/{id}
pub async fn get_site(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult {
    check_licence(&state)?;
    let site = load_site(&state.storage, id).await?;
    site_detail(&state.storage, site).await
}

/// POST /api/v1/energy/sites
pub async fn create_site(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(input): Json<SiteInput>,
) -> ApiResult {
    check_licence(&state)?;
    let v = merge_site(&input, None)?;
    let clock = LocalClock::parse(&v.timezone).map_err(AppError::bad_request)?;
    let tariff = match input.tariff {
        Some(t) => t,
        None => TariffInput {
            valid_from: clock.month_of(now()),
            values: TariffValues::defaults(Golongan::I3),
            note: String::new(),
        },
    };
    tariff.validate()?;
    let ts = now();
    let name = v.name.clone();
    let id = db(&state.storage, move |c| {
        let tx = c.unchecked_transaction()?;
        let id = store::insert_site(
            &tx,
            &SiteFields {
                name: &v.name,
                timezone: &v.timezone,
                wbp_windows: &v.windows,
                contracted_kva: v.kva,
                alerts: &v.alerts,
            },
            ts,
        )?;
        store::upsert_tariff(
            &tx,
            id,
            &tariff.valid_from,
            &tariff.values,
            &tariff.note,
            ts,
        )?;
        tx.commit()?;
        Ok(id)
    })
    .await?;
    audit(
        &state,
        &auth,
        "energy_site_create",
        format!("energy_site:{id}"),
        name,
    );
    let site = load_site(&state.storage, id).await?;
    site_detail(&state.storage, site).await
}

/// PUT /api/v1/energy/sites/{id}
pub async fn update_site(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(input): Json<SiteInput>,
) -> ApiResult {
    check_licence(&state)?;
    let current = load_site(&state.storage, id).await?;
    let v = merge_site(&input, Some(&current))?;
    let reband = v.timezone != current.timezone || v.windows != current.wbp_windows;
    let ts = now();
    let details = format!(
        "name={}, timezone={}, wbp={:?}",
        v.name, v.timezone, v.windows
    );
    let site = db(&state.storage, move |c| {
        store::update_site(
            c,
            id,
            &SiteFields {
                name: &v.name,
                timezone: &v.timezone,
                wbp_windows: &v.windows,
                contracted_kva: v.kva,
                alerts: &v.alerts,
            },
            ts,
        )?;
        let site = store::get_site(c, id)?.ok_or_else(|| anyhow::anyhow!("site vanished"))?;
        if reband {
            store::reband_site(c, &site)?;
        }
        Ok(site)
    })
    .await?;
    audit(
        &state,
        &auth,
        "energy_site_update",
        format!("energy_site:{id}"),
        details,
    );
    site_detail(&state.storage, site).await
}

/// DELETE /api/v1/energy/sites/{id}
pub async fn delete_site(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> ApiResult {
    check_licence(&state)?;
    if !db(&state.storage, move |c| store::delete_site(c, id)).await? {
        return Err(AppError::not_found("Energy site not found"));
    }
    audit(
        &state,
        &auth,
        "energy_site_delete",
        format!("energy_site:{id}"),
        String::new(),
    );
    Ok(Json(json!({"deleted": id})))
}

// ── Tariffs ──

/// POST /api/v1/energy/sites/{id}/tariffs (replaces the tariff of the same `valid_from`)
pub async fn upsert_tariff(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(site_id): Path<i64>,
    Json(input): Json<TariffInput>,
) -> ApiResult {
    check_licence(&state)?;
    input.validate()?;
    load_site(&state.storage, site_id).await?;
    let details = format!(
        "{} from {}",
        input.values.golongan.as_str(),
        input.valid_from
    );
    let id = db(&state.storage, move |c| {
        store::upsert_tariff(
            c,
            site_id,
            &input.valid_from,
            &input.values,
            &input.note,
            now(),
        )
    })
    .await?;
    audit(
        &state,
        &auth,
        "energy_tariff_set",
        format!("energy_tariff:{id}"),
        details,
    );
    let t = db(&state.storage, move |c| store::get_tariff(c, id)).await?;
    Ok(Json(json!(t)))
}

/// PUT /api/v1/energy/tariffs/{id}
pub async fn update_tariff(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(input): Json<TariffInput>,
) -> ApiResult {
    check_licence(&state)?;
    input.validate()?;
    let details = format!(
        "{} from {}",
        input.values.golongan.as_str(),
        input.valid_from
    );
    let updated = db(&state.storage, move |c| {
        store::update_tariff(c, id, &input.valid_from, &input.values, &input.note)
    })
    .await
    .map_err(|e| {
        if format!("{:#}", e.0).contains("UNIQUE") {
            AppError::bad_request("the site already has a tariff starting that month")
        } else {
            e
        }
    })?;
    if !updated {
        return Err(AppError::not_found("Tariff not found"));
    }
    audit(
        &state,
        &auth,
        "energy_tariff_set",
        format!("energy_tariff:{id}"),
        details,
    );
    let t = db(&state.storage, move |c| store::get_tariff(c, id)).await?;
    Ok(Json(json!(t)))
}

/// DELETE /api/v1/energy/tariffs/{id} (a site keeps at least one)
pub async fn delete_tariff(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> ApiResult {
    check_licence(&state)?;
    let t: Tariff = db(&state.storage, move |c| store::get_tariff(c, id))
        .await?
        .ok_or_else(|| AppError::not_found("Tariff not found"))?;
    let site_id = t.site_id;
    let count = db(&state.storage, move |c| {
        Ok(store::list_tariffs(c, site_id)?.len())
    })
    .await?;
    if count <= 1 {
        return Err(AppError::bad_request("a site needs at least one tariff"));
    }
    db(&state.storage, move |c| store::delete_tariff(c, id)).await?;
    audit(
        &state,
        &auth,
        "energy_tariff_delete",
        format!("energy_tariff:{id}"),
        t.valid_from,
    );
    Ok(Json(json!({"deleted": id})))
}

// ── Meters ──

#[derive(Debug, Deserialize)]
pub struct MeterInput {
    pub name: String,
    pub device_id: String,
    pub kwh_metric: String,
    #[serde(default)]
    pub kvarh_metric: Option<String>,
    #[serde(default)]
    pub kvar_metric: Option<String>,
    pub role: MeterRole,
    #[serde(default = "one")]
    pub multiplier: f64,
    #[serde(default)]
    pub rollover: Option<f64>,
    #[serde(default = "default_max_gap")]
    pub max_gap_secs: i64,
    #[serde(default = "yes")]
    pub enabled: bool,
}

fn one() -> f64 {
    1.0
}
fn default_max_gap() -> i64 {
    900
}
fn yes() -> bool {
    true
}

fn blank_to_none(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn meter_from_input(input: MeterInput, id: i64, site_id: i64) -> Result<Meter, AppError> {
    let name = input.name.trim().to_string();
    if name.is_empty() || name.len() > 100 {
        return Err(AppError::bad_request(
            "name is required (at most 100 characters)",
        ));
    }
    let device_id = input.device_id.trim().to_string();
    let kwh_metric = input.kwh_metric.trim().to_string();
    if device_id.is_empty() || kwh_metric.is_empty() {
        return Err(AppError::bad_request(
            "device_id and kwh_metric are required",
        ));
    }
    if !(input.multiplier.is_finite() && input.multiplier > 0.0) {
        return Err(AppError::bad_request("multiplier must be positive"));
    }
    if let Some(r) = input.rollover
        && !(r.is_finite() && r > 0.0)
    {
        return Err(AppError::bad_request("rollover must be positive"));
    }
    if !(60..=86_400).contains(&input.max_gap_secs) {
        return Err(AppError::bad_request("max_gap_secs must be 60 to 86400"));
    }
    Ok(Meter {
        id,
        site_id,
        name,
        device_id,
        kwh_metric,
        kvarh_metric: blank_to_none(input.kvarh_metric),
        kvar_metric: blank_to_none(input.kvar_metric),
        role: input.role,
        multiplier: input.multiplier,
        rollover: input.rollover,
        max_gap_secs: input.max_gap_secs,
        enabled: input.enabled,
        created_at: 0,
        updated_at: 0,
    })
}

/// Compute a meter's hours from the start of the previous billing month, in
/// the background (after it was added or its tags changed).
fn backfill(storage: Storage, site: Site, meter: Meter) {
    tokio::spawn(async move {
        let t = now();
        let clock = site.clock();
        let Ok((this_month, _)) = clock.month_range(&clock.month_of(t)) else {
            return;
        };
        let Ok((from, _)) = clock.month_range(&clock.month_of(this_month - 1)) else {
            return;
        };
        let to = t.div_euclid(3600) * 3600;
        if let Err(e) = job::compute_meter(&storage, &site, &meter, from, to, t).await {
            tracing::warn!("Energy: backfill of meter {} failed: {e:#}", meter.id);
        }
    });
}

/// POST /api/v1/energy/sites/{id}/meters
pub async fn create_meter(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(site_id): Path<i64>,
    Json(input): Json<MeterInput>,
) -> ApiResult {
    check_licence(&state)?;
    let site = load_site(&state.storage, site_id).await?;
    let mut meter = meter_from_input(input, 0, site_id)?;
    let m = meter.clone();
    let ts = now();
    meter.id = db(&state.storage, move |c| store::insert_meter(c, &m, ts)).await?;
    audit(
        &state,
        &auth,
        "energy_meter_create",
        format!("energy_meter:{}", meter.id),
        format!("{} {}/{}", meter.name, meter.device_id, meter.kwh_metric),
    );
    if meter.enabled {
        backfill(state.storage.clone(), site, meter.clone());
    }
    let id = meter.id;
    Ok(Json(json!(
        db(&state.storage, move |c| store::get_meter(c, id)).await?
    )))
}

/// PUT /api/v1/energy/meters/{id}
pub async fn update_meter(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(input): Json<MeterInput>,
) -> ApiResult {
    check_licence(&state)?;
    let current = db(&state.storage, move |c| store::get_meter(c, id))
        .await?
        .ok_or_else(|| AppError::not_found("Meter not found"))?;
    let site = load_site(&state.storage, current.site_id).await?;
    let meter = meter_from_input(input, id, current.site_id)?;
    let recompute = meter.enabled
        && (meter.device_id != current.device_id
            || meter.kwh_metric != current.kwh_metric
            || meter.kvarh_metric != current.kvarh_metric
            || meter.kvar_metric != current.kvar_metric
            || meter.multiplier != current.multiplier
            || meter.rollover != current.rollover
            || meter.max_gap_secs != current.max_gap_secs
            || !current.enabled);
    let m = meter.clone();
    let ts = now();
    db(&state.storage, move |c| store::update_meter(c, &m, ts)).await?;
    audit(
        &state,
        &auth,
        "energy_meter_update",
        format!("energy_meter:{id}"),
        format!(
            "{} {}/{} x{}",
            meter.name, meter.device_id, meter.kwh_metric, meter.multiplier
        ),
    );
    if recompute {
        backfill(state.storage.clone(), site, meter);
    }
    Ok(Json(json!(
        db(&state.storage, move |c| store::get_meter(c, id)).await?
    )))
}

/// DELETE /api/v1/energy/meters/{id}
pub async fn delete_meter(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> ApiResult {
    check_licence(&state)?;
    if !db(&state.storage, move |c| store::delete_meter(c, id)).await? {
        return Err(AppError::not_found("Meter not found"));
    }
    audit(
        &state,
        &auth,
        "energy_meter_delete",
        format!("energy_meter:{id}"),
        String::new(),
    );
    Ok(Json(json!({"deleted": id})))
}

#[derive(Debug, Deserialize)]
pub struct HoursQuery {
    pub from: Option<i64>,
    pub to: Option<i64>,
}

/// GET /api/v1/energy/meters/{id}/hours?from=&to= (UTC seconds, default the last 48 h)
pub async fn meter_hours(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(q): Query<HoursQuery>,
) -> ApiResult {
    check_licence(&state)?;
    let to = q.to.unwrap_or_else(now);
    let from = q.from.unwrap_or(to - 48 * 3600);
    if to - from > 62 * 86_400 || to < from {
        return Err(AppError::bad_request("from/to must span at most 62 days"));
    }
    let rows = db(&state.storage, move |c| {
        store::hours_for_meters(c, &[id], from, to)
    })
    .await?;
    Ok(Json(json!(rows)))
}

// ── Bill ──

#[derive(Debug, Deserialize)]
pub struct MonthQuery {
    pub month: Option<String>,
}

fn month_or_current(site: &Site, q: &MonthQuery) -> Result<String, AppError> {
    match &q.month {
        Some(m) => {
            parse_month(m).map_err(AppError::bad_request)?;
            Ok(m.trim().to_string())
        }
        None => Ok(site.clock().month_of(now())),
    }
}

/// Incomer meters, their rows for the month, the tariffs and PLN's figures.
async fn month_report(
    storage: &Storage,
    site: &Site,
    month: &str,
) -> Result<MonthReport, AppError> {
    let (from, to) = site
        .clock()
        .month_range(month)
        .map_err(AppError::bad_request)?;
    let (site_id, m) = (site.id, month.to_string());
    let (tariffs, incomers, rows, stored) = db(storage, move |c| {
        let tariffs = store::list_tariffs(c, site_id)?;
        let incomers: Vec<i64> = store::list_meters(c, Some(site_id))?
            .into_iter()
            .filter(|m| m.enabled && m.role == MeterRole::PlnIncomer)
            .map(|m| m.id)
            .collect();
        let rows = store::hours_for_meters(c, &incomers, from, to)?;
        let stored = store::list_bills(c, site_id)?
            .into_iter()
            .find(|b| b.month == m);
        Ok((tariffs, incomers, rows, stored))
    })
    .await?;
    let mut report = report::month_report(site, &tariffs, month, &rows, incomers, now())
        .map_err(AppError::bad_request)?;
    if let Some(b) = stored
        && let Ok(pln) = serde_json::from_value::<PlnFigures>(b.pln)
    {
        report.attach_pln(pln);
    }
    Ok(report)
}

/// GET /api/v1/energy/sites/{id}/bill?month=YYYY-MM
pub async fn get_bill(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(q): Query<MonthQuery>,
) -> ApiResult {
    check_licence(&state)?;
    let site = load_site(&state.storage, id).await?;
    let month = month_or_current(&site, &q)?;
    let mut report = month_report(&state.storage, &site, &month).await?;
    report.confirmed = site_confirmed(&state.storage, &site).await?;
    let mut body = json!(report);
    body["unconfirmed"] = unconfirmed();
    body["notification_keys"] = site_keys(id);
    Ok(Json(body))
}

/// GET /api/v1/energy/sites/{id}/daily?month=YYYY-MM
pub async fn get_daily(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(q): Query<MonthQuery>,
) -> ApiResult {
    check_licence(&state)?;
    let site = load_site(&state.storage, id).await?;
    let month = month_or_current(&site, &q)?;
    let (from, to) = site
        .clock()
        .month_range(&month)
        .map_err(AppError::bad_request)?;
    let (tariffs, rows) = db(&state.storage, move |c| {
        let incomers: Vec<i64> = store::list_meters(c, Some(id))?
            .into_iter()
            .filter(|m| m.enabled && m.role == MeterRole::PlnIncomer)
            .map(|m| m.id)
            .collect();
        Ok((
            store::list_tariffs(c, id)?,
            store::hours_for_meters(c, &incomers, from, to)?,
        ))
    })
    .await?;
    Ok(Json(json!({
        "month": month,
        "days": report::daily(&site, &tariffs, &month, &rows),
    })))
}

// ── Reconciliation ──

/// GET /api/v1/energy/sites/{id}/reconcile
pub async fn list_reconciled(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult {
    check_licence(&state)?;
    let site = load_site(&state.storage, id).await?;
    let bills = db(&state.storage, move |c| store::list_bills(c, id)).await?;
    let mut out = Vec::new();
    for b in bills {
        let report = month_report(&state.storage, &site, &b.month).await?;
        out.push(json!({
            "month": b.month,
            "pln": b.pln,
            "computed_at_entry": b.computed,
            "entered_by": b.entered_by,
            "entered_at": b.entered_at,
            "reconciliation": report.reconciliation,
        }));
    }
    Ok(Json(json!(out)))
}

/// PUT /api/v1/energy/sites/{id}/reconcile/{month}
pub async fn put_reconcile(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((id, month)): Path<(i64, String)>,
    Json(pln): Json<PlnFigures>,
) -> ApiResult {
    check_licence(&state)?;
    parse_month(&month).map_err(AppError::bad_request)?;
    pln.validate().map_err(AppError::bad_request)?;
    let site = load_site(&state.storage, id).await?;
    let report = month_report(&state.storage, &site, &month).await?;
    let stored = StoredBill {
        site_id: id,
        month: month.clone(),
        pln: json!(pln),
        computed: json!(report.month_to_date),
        entered_by: auth.username.clone(),
        entered_at: now(),
    };
    db(&state.storage, move |c| store::upsert_bill(c, &stored)).await?;
    audit(
        &state,
        &auth,
        "energy_reconcile",
        format!("energy_site:{id}"),
        format!(
            "month={month}, kwh_lwbp={}, kwh_wbp={}, kvarh={}",
            pln.kwh_lwbp, pln.kwh_wbp, pln.kvarh
        ),
    );
    let mut report = month_report(&state.storage, &site, &month).await?;
    report.confirmed = site_confirmed(&state.storage, &site).await?;
    Ok(Json(json!(report)))
}

/// DELETE /api/v1/energy/sites/{id}/reconcile/{month}
pub async fn delete_reconcile(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((id, month)): Path<(i64, String)>,
) -> ApiResult {
    check_licence(&state)?;
    let m = month.clone();
    if !db(&state.storage, move |c| store::delete_bill(c, id, &m)).await? {
        return Err(AppError::not_found("No PLN figures for that month"));
    }
    audit(
        &state,
        &auth,
        "energy_reconcile_delete",
        format!("energy_site:{id}"),
        month,
    );
    Ok(Json(json!({"deleted": true})))
}

/// POST /api/v1/energy/sites/{id}/recompute?month=YYYY-MM
pub async fn recompute(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i64>,
    Query(q): Query<MonthQuery>,
) -> ApiResult {
    check_licence(&state)?;
    let site = load_site(&state.storage, id).await?;
    let month = month_or_current(&site, &q)?;
    let rows = job::recompute_month(&state.storage, &site, &month, now()).await?;
    audit(
        &state,
        &auth,
        "energy_recompute",
        format!("energy_site:{id}"),
        month.clone(),
    );
    Ok(Json(json!({"month": month, "rows": rows})))
}
