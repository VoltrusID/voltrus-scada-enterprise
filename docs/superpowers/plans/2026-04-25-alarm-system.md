# Alarm System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add threshold-based alarms with looping audio, persistent alarm history, acknowledgment UI, and Web Push groundwork.

**Architecture:** Backend evaluates thresholds in the persistence task loop, writes alarm records to SQLite. Frontend detects breaches from existing `/api/data` responses for real-time audio and badge count. Alarm list panel fetches `/api/alarms` lazily when opened. Web Push stubs added for future implementation.

**Tech Stack:** Rust/Axum/SQLite (backend), React/TypeScript (frontend), Web Audio API (audio playback), Service Workers (push groundwork)

---

## File Structure

### Backend (Rust)

| Action | File | Responsibility |
|--------|------|---------------|
| Modify | `src/models.rs` | Add `Alarm` struct, `AlarmQuery` |
| Modify | `src/storage.rs` | Add alarms table to schema, CRUD methods |
| Create | `src/api/alarms.rs` | Alarm API handlers (list, acknowledge, acknowledge-all) |
| Modify | `src/api/mod.rs` | Register alarm routes, add `alarms` module |
| Modify | `src/main.rs` | Add alarm evaluation in persistence task |

### Frontend (TypeScript/React)

| Action | File | Responsibility |
|--------|------|---------------|
| Modify | `client/src/types.ts` | Add `Alarm` interface |
| Modify | `client/src/queries.ts` | Add alarm query/mutation hooks |
| Create | `client/src/AlarmPanel.tsx` | Side panel UI with alarm list |
| Create | `client/src/useAlarmAudio.ts` | Audio playback hook |
| Modify | `client/src/App.tsx` | Integrate alarm badge, audio hook, panel toggle |
| Create | `client/public/sw.js` | Service worker shell for push |

### Audio

| Action | File | Responsibility |
|--------|------|---------------|
| Create | `client/public/alarm.mp3` | Placeholder — user will provide actual audio |

---

### Task 1: Add Alarm data model

**Files:**
- Modify: `src/models.rs:268` (after `DataQuery` struct)

- [ ] **Step 1: Add `Alarm` and `AlarmQuery` structs to `src/models.rs`**

Add after the `DataQuery` struct (after line 335):

```rust
// ── Alarm types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Alarm {
    pub id: i64,
    pub source_key: String,
    pub source_name: String,
    pub severity: String,
    pub value: f64,
    pub threshold: f64,
    pub message: String,
    pub triggered_at: i64,
    pub resolved_at: Option<i64>,
    pub acknowledged_at: Option<i64>,
    pub acknowledged_by: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct AlarmQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check 2>&1 | head -20`
Expected: No errors related to the new types.

- [ ] **Step 3: Commit**

```bash
git add src/models.rs
git commit -m "feat(alarm): add Alarm and AlarmQuery data models"
```

---

### Task 2: Add alarms table and storage methods

**Files:**
- Modify: `src/storage.rs:837` (in `init_schema` function)

- [ ] **Step 1: Add `alarms` table to `init_schema` in `src/storage.rs`**

Add the following SQL inside the `conn.execute_batch(` call in `init_schema`, after the `twin_layouts` table definition (before the closing `"`)`):

```sql
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
```

- [ ] **Step 2: Add storage methods for alarms**

Add the following methods to the `impl Storage` block in `src/storage.rs` (after the last existing method, before the closing `}` of the impl block). Also add `crate::models::Alarm` and `crate::models::AlarmQuery` to the imports at the top.

Update imports at line 6-8 to include `Alarm` and `AlarmQuery`:

```rust
use crate::models::{
    DataPoint, Device, ScreenLayoutRow, DataSource, Setting, TwinLayout, User, UserShortcut,
    Alarm, AlarmQuery,
};
```

Add these methods to `impl Storage`:

```rust
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
            .context("Insert alarm failed")
        })
        .await
        .context("Task join failed")?
        .map(|_| ())
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
                        source_name: String::new(), // resolved in API handler
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
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check 2>&1 | head -20`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src/storage.rs
git commit -m "feat(alarm): add alarms table schema and storage CRUD methods"
```

---

### Task 3: Add alarm API endpoints

**Files:**
- Create: `src/api/alarms.rs`
- Modify: `src/api/mod.rs:1` (add module), `src/api/mod.rs:81` (add routes)

- [ ] **Step 1: Create `src/api/alarms.rs`**

```rust
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::models::*;
use super::{AppError, AppState, AuthUser};

/// GET /api/alarms?status=active|acknowledged|all&limit=50
pub async fn list_alarms(
    State(state): State<AppState>,
    Query(params): Query<AlarmQuery>,
) -> Result<impl IntoResponse, AppError> {
    let status = params.status.as_deref();
    let limit = params.limit.unwrap_or(50);
    let mut alarms = state.storage.list_alarms(status, limit).await?;

    // Resolve source_name from data_sources
    let sources = state.storage.list_data_sources(None).await?;
    let name_map: std::collections::HashMap<String, String> = sources
        .iter()
        .map(|s| (s.key.clone(), s.name.clone()))
        .collect();

    for alarm in &mut alarms {
        if let Some(name) = name_map.get(&alarm.source_key) {
            alarm.source_name = name.clone();
        } else {
            alarm.source_name = alarm.source_key.clone();
        }
    }

    Ok(Json(alarms))
}

/// POST /api/alarms/{id}/acknowledge
pub async fn acknowledge_alarm(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    state.storage.acknowledge_alarm(id, &user.username).await?;
    Ok(Json(json!({"ok": true})))
}

/// POST /api/alarms/acknowledge-all
pub async fn acknowledge_all(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    state.storage.acknowledge_all(&user.username).await?;
    Ok(Json(json!({"ok": true})))
}
```

- [ ] **Step 2: Register the module and routes in `src/api/mod.rs`**

At line 1-9, add `alarms` to the module declarations:

```rust
pub(crate) mod auth;
mod screens;
mod data_sources;
mod layouts;
mod shortcuts;
mod settings;
mod data;
mod system;
mod generators;
mod alarms;
```

In the `protected` router (after the shortcuts route at ~line 127, before `.layer(middleware...)`):

```rust
        .route(
            "/api/alarms",
            get(alarms::list_alarms).post(alarms::acknowledge_all),
        )
        .route(
            "/api/alarms/{id}/acknowledge",
            post(alarms::acknowledge_alarm),
        )
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check 2>&1 | head -30`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src/api/alarms.rs src/api/mod.rs
git commit -m "feat(alarm): add alarm API endpoints (list, acknowledge, acknowledge-all)"
```

---

### Task 4: Add alarm evaluation to backend persistence loop

**Files:**
- Modify: `src/main.rs:49-77` (persistence task)

- [ ] **Step 1: Add alarm evaluation logic to the persistence task in `src/main.rs`**

The persistence task already iterates over snapshots. After the existing `storage.insert_batch` call (line 72), add alarm evaluation. The full updated persistence task block becomes:

```rust
    // Spawn persistence task
    {
        let cache = cache.clone();
        let storage = storage.clone();
        let interval = app_config.persist_interval_secs;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval));
            loop {
                ticker.tick().await;
                let snapshots = cache.get_all().await;
                if snapshots.is_empty() {
                    continue;
                }
                let mut records = Vec::new();
                for snap in &snapshots {
                    for (metric, value) in &snap.metrics {
                        records.push((
                            snap.device_id.clone(),
                            snap.timestamp,
                            metric.clone(),
                            *value,
                        ));
                    }
                }
                if let Err(e) = storage.insert_batch(&records).await {
                    tracing::error!("Persistence error: {e}");
                }

                // Evaluate alarm thresholds
                if let Ok(sources) = storage.list_data_sources(None).await {
                    // Build a lookup: (device_id, metric_key) -> value from cache
                    let mut value_map = std::collections::HashMap::new();
                    for snap in &snapshots {
                        for (metric, value) in &snap.metrics {
                            value_map.insert(
                                format!("{}:{}", snap.device_id, metric),
                                *value,
                            );
                        }
                    }

                    for source in &sources {
                        let value = if source.source_type == "modbus" {
                            *value_map
                                .get(&format!("{}:{}", source.device_id, source.key))
                                .unwrap_or(&source.base_val)
                        } else {
                            // For simulated sources, use base_val as we don't re-generate here
                            // The frontend detects these from /api/data
                            continue;
                        };

                        if source.crit_val > 0.0 && value >= source.crit_val {
                            if !storage.has_active_alarm(&source.key, "critical").await.unwrap_or(false) {
                                let msg = format!(
                                    "{} reached {}{} (critical: {}{})",
                                    source.name, value, source.unit, source.crit_val, source.unit
                                );
                                let _ = storage.create_alarm(&source.key, "critical", value, source.crit_val, &msg).await;
                            }
                        } else if source.warn_val > 0.0 && value >= source.warn_val {
                            if !storage.has_active_alarm(&source.key, "warning").await.unwrap_or(false) {
                                let msg = format!(
                                    "{} reached {}{} (warning: {}{})",
                                    source.name, value, source.unit, source.warn_val, source.unit
                                );
                                let _ = storage.create_alarm(&source.key, "warning", value, source.warn_val, &msg).await;
                            }
                        } else {
                            // Value is normal — resolve any acknowledged alarms
                            let _ = storage.resolve_active_alarms(&source.key).await;
                        }
                    }
                }
            }
        });
    }
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check 2>&1 | head -30`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat(alarm): evaluate thresholds in persistence loop, create/resolve alarms"
```

---

### Task 5: Add frontend Alarm type and query hooks

**Files:**
- Modify: `client/src/types.ts:29` (after `EquipmentType`)
- Modify: `client/src/queries.ts:17` (add alarm keys and hooks)

- [ ] **Step 1: Add `Alarm` interface to `client/src/types.ts`**

Add after the `EQUIPMENT_TYPES` export (after line 52):

```ts
export interface Alarm {
  id: number
  source_key: string
  source_name: string
  severity: 'warning' | 'critical'
  value: number
  threshold: number
  message: string
  triggered_at: number
  resolved_at: number | null
  acknowledged_at: number | null
  acknowledged_by: string | null
}
```

- [ ] **Step 2: Add alarm query keys and hooks to `client/src/queries.ts`**

Add to the `queryKeys` object (after `systemInfo`):

```ts
  alarms:       (status: string) => ['alarms', status] as const,
```

Add to imports at top — add `Alarm` to the types import:

```ts
import type { SensorConfig, Screen, ScreenLayouts, DataPoint, Alarm } from './types'
```

Add query and mutation hooks at the end of the file:

```ts
// ── Alarm hooks ─────────────────────────────────────────────────────

export function useAlarms(status: string = 'all', enabled: boolean = true) {
  return useQuery({
    queryKey: queryKeys.alarms(status),
    queryFn: () => fetchJson<Alarm[]>(`/api/alarms?status=${status}&limit=50`),
    enabled,
    staleTime: 5_000,
  })
}

export function useAcknowledgeAlarm() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: number) => {
      await fetchOk(`/api/alarms/${id}/acknowledge`, { method: 'POST' })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['alarms'] })
    },
  })
}

export function useAcknowledgeAllAlarms() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async () => {
      await fetchOk('/api/alarms', { method: 'POST' })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['alarms'] })
    },
  })
}
```

- [ ] **Step 3: Verify TypeScript compiles**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`
Expected: No errors related to new types/hooks.

- [ ] **Step 4: Commit**

```bash
git add client/src/types.ts client/src/queries.ts
git commit -m "feat(alarm): add Alarm type and alarm query/mutation hooks"
```

---

### Task 6: Create alarm audio hook

**Files:**
- Create: `client/src/useAlarmAudio.ts`

- [ ] **Step 1: Create `client/src/useAlarmAudio.ts`**

```ts
import { useRef, useCallback, useEffect } from 'react'

export function useAlarmAudio() {
  const audioRef = useRef<HTMLAudioElement | null>(null)
  const mutedRef = useRef(false)
  const playingRef = useRef(false)

  useEffect(() => {
    audioRef.current = new Audio('/alarm.mp3')
    audioRef.current.loop = true
    audioRef.current.volume = 0.5
    return () => {
      if (audioRef.current) {
        audioRef.current.pause()
        audioRef.current = null
      }
    }
  }, [])

  const startAlarm = useCallback(() => {
    if (!audioRef.current || mutedRef.current || playingRef.current) return
    audioRef.current.play().catch(() => {})
    playingRef.current = true
  }, [])

  const stopAlarm = useCallback(() => {
    if (!audioRef.current) return
    audioRef.current.pause()
    audioRef.current.currentTime = 0
    playingRef.current = false
  }, [])

  const toggleMute = useCallback(() => {
    mutedRef.current = !mutedRef.current
    if (mutedRef.current) stopAlarm()
  }, [stopAlarm])

  const isMuted = useCallback(() => mutedRef.current, [])

  return { startAlarm, stopAlarm, toggleMute, isMuted }
}
```

- [ ] **Step 2: Create a placeholder alarm audio file**

Create `client/public/alarm.mp3` as a placeholder. Since we can't generate real audio, create an empty marker file:

```bash
touch client/public/alarm.mp3
```

The user will replace this with the actual alarm sound file.

- [ ] **Step 3: Commit**

```bash
git add client/src/useAlarmAudio.ts client/public/alarm.mp3
git commit -m "feat(alarm): add useAlarmAudio hook with loop/mute support"
```

---

### Task 7: Create alarm side panel component

**Files:**
- Create: `client/src/AlarmPanel.tsx`

- [ ] **Step 1: Create `client/src/AlarmPanel.tsx`**

```tsx
import { useState } from 'react'
import { useAlarms, useAcknowledgeAlarm, useAcknowledgeAllAlarms } from './queries'
import { fmtTs } from './styles'

const FILTER_TABS = ['all', 'active', 'acknowledged'] as const

interface Props {
  onClose: () => void
}

export function AlarmPanel({ onClose }: Props) {
  const [filter, setFilter] = useState<string>('active')
  const { data: alarms = [], isLoading } = useAlarms(filter)
  const ackMut = useAcknowledgeAlarm()
  const ackAllMut = useAcknowledgeAllAlarms()

  return (
    <div style={{ position: 'fixed', inset: 0, zIndex: 1000, display: 'flex', justifyContent: 'flex-end' }}>
      <div onClick={onClose} style={{ flex: 1, background: 'rgba(0,0,0,.5)' }} />
      <div style={{ width: 360, height: '100%', background: 'var(--bg)', borderLeft: '1px solid var(--border)', display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        {/* Header */}
        <div style={{ padding: '16px 16px 12px', borderBottom: '1px solid var(--border)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: 15, fontWeight: 700 }}>Alarms</span>
          <div style={{ display: 'flex', gap: 8 }}>
            <button onClick={() => ackAllMut.mutate()} style={{ padding: '4px 12px', borderRadius: 6, border: '1px solid var(--warn)', background: 'transparent', color: 'var(--warn)', cursor: 'pointer', fontSize: 11, fontWeight: 600 }}>Ack All</button>
            <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--muted)', cursor: 'pointer', fontSize: 18, padding: '0 4px' }}>&times;</button>
          </div>
        </div>

        {/* Filter tabs */}
        <div style={{ display: 'flex', borderBottom: '1px solid var(--border)' }}>
          {FILTER_TABS.map(tab => (
            <button key={tab} onClick={() => setFilter(tab)} style={{
              flex: 1, padding: '8px 0', fontSize: 11, fontWeight: 600, textTransform: 'uppercase',
              letterSpacing: '.04em', cursor: 'pointer', border: 'none', background: 'transparent',
              borderBottom: filter === tab ? '2px solid var(--cyan)' : '2px solid transparent',
              color: filter === tab ? 'var(--cyan)' : 'var(--muted)',
            }}>{tab}</button>
          ))}
        </div>

        {/* Alarm list */}
        <div style={{ flex: 1, overflow: 'auto', padding: 8 }}>
          {isLoading && <div style={{ padding: 20, color: 'var(--muted)', textAlign: 'center', fontSize: 13 }}>Loading...</div>}
          {!isLoading && alarms.length === 0 && <div style={{ padding: 20, color: 'var(--muted)', textAlign: 'center', fontSize: 13 }}>No alarms</div>}
          {alarms.map(alarm => {
            const isCritical = alarm.severity === 'critical'
            const isActive = !alarm.acknowledged_at
            const isResolved = !!alarm.resolved_at
            const sevColor = isCritical ? 'var(--crit)' : 'var(--warn)'
            const sevBg = isCritical ? 'rgba(239,68,68,.15)' : 'rgba(234,179,8,.15)'
            return (
              <div key={alarm.id} style={{
                padding: 12, marginBottom: 8, borderRadius: 8,
                border: `1px solid ${isActive ? sevColor : 'var(--border)'}`,
                background: isActive ? sevBg : 'var(--card)',
                opacity: isResolved ? 0.6 : 1,
              }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                    {isActive && <div style={{ width: 8, height: 8, borderRadius: '50%', background: sevColor, animation: 'pulse 1.5s infinite' }} />}
                    <span style={{ padding: '2px 8px', borderRadius: 4, fontSize: 10, fontWeight: 700, textTransform: 'uppercase', background: sevBg, color: sevColor }}>{alarm.severity}</span>
                  </div>
                  <span style={{ fontSize: 11, color: 'var(--muted)' }}>{fmtTs(alarm.triggered_at * 1000)}</span>
                </div>
                <div style={{ fontSize: 13, fontWeight: 600, marginBottom: 4 }}>{alarm.source_name}</div>
                <div style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 8 }}>{alarm.message}</div>
                {isActive && (
                  <button onClick={() => ackMut.mutate(alarm.id)} style={{
                    padding: '4px 12px', borderRadius: 6, fontSize: 11, fontWeight: 600, cursor: 'pointer',
                    border: `1px solid ${sevColor}`, background: 'transparent', color: sevColor,
                  }}>Acknowledge</button>
                )}
                {alarm.acknowledged_at && (
                  <span style={{ fontSize: 11, color: 'var(--ok)' }}>&#10003; Acknowledged{alarm.acknowledged_by ? ` by ${alarm.acknowledged_by}` : ''}</span>
                )}
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Add the pulse animation CSS**

Add a `<style>` tag injection in `AlarmPanel.tsx` — add this at the top of the component, before the return:

```tsx
// Inject pulse animation once
if (typeof document !== 'undefined' && !document.getElementById('alarm-pulse-style')) {
  const style = document.createElement('style')
  style.id = 'alarm-pulse-style'
  style.textContent = '@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }'
  document.head.appendChild(style)
}
```

- [ ] **Step 3: Commit**

```bash
git add client/src/AlarmPanel.tsx
git commit -m "feat(alarm): add AlarmPanel side panel with filter, acknowledge UI"
```

---

### Task 8: Integrate alarm system into App.tsx

**Files:**
- Modify: `client/src/App.tsx`

- [ ] **Step 1: Add imports at the top of `client/src/App.tsx`**

Add to the existing import block:

```ts
import { useAlarmAudio } from './useAlarmAudio'
import { AlarmPanel } from './AlarmPanel'
```

- [ ] **Step 2: Add alarm state and audio hook in App component**

After the existing `const [alerts, setAlerts] = useState(0);` line (~line 30), add:

```ts
  const [showAlarmPanel, setShowAlarmPanel] = useState(false);
  const [alarmMuted, setAlarmMuted] = useState(false);
  const { startAlarm, stopAlarm, toggleMute, isMuted } = useAlarmAudio();
```

- [ ] **Step 3: Replace the existing alert counter with alarm detection logic**

In the rAF loop's `pendingData.current` block (around line 267-284), replace the alert counting with alarm breach tracking. Replace the existing block:

```ts
        let a = 0;
        for (const item of d) {
          const arr = newMap[item.key] ? [...newMap[item.key]] : [];
          arr.push({ value: item.value, ts: Date.now() });
          if (arr.length > (turbo ? MAX_PTS : NORM_PTS)) arr.shift();
          newMap[item.key] = arr;
          newVals[item.key] = item;
          if (item.value >= item.crit || item.value >= item.warn) a++;
        }
        setDataMap(newMap);
        setCurrentValues(newVals);
        setAlerts(a);
```

With:

```ts
        let alertCount = 0;
        const newBreaches = new Set<string>();
        for (const item of d) {
          const arr = newMap[item.key] ? [...newMap[item.key]] : [];
          arr.push({ value: item.value, ts: Date.now() });
          if (arr.length > (turbo ? MAX_PTS : NORM_PTS)) arr.shift();
          newMap[item.key] = arr;
          newVals[item.key] = item;
          if (item.value >= item.crit || item.value >= item.warn) {
            alertCount++;
            newBreaches.add(item.key);
          }
        }
        setDataMap(newMap);
        setCurrentValues(newVals);
        setAlerts(alertCount);

        // Alarm audio: start on new breaches, stop when all clear
        if (newBreaches.size > 0) {
          startAlarm();
        } else {
          stopAlarm();
        }
```

- [ ] **Step 4: Add alarm bell button to the header bar**

In the header bar button row (the `<div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>` at ~line 356), add the alarm bell and mute buttons before the existing view mode toggle. Add as the first child of that div:

```tsx
            {/* Alarm bell */}
            <button onClick={() => setShowAlarmPanel(true)} style={{
              ...btnStyle, position: 'relative', borderColor: alerts > 0 ? 'var(--crit)' : 'var(--border)',
              color: alerts > 0 ? 'var(--crit)' : 'var(--fg)',
            }}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
              {alerts > 0 && <span style={{
                position: 'absolute', top: -4, right: -4, background: 'var(--crit)',
                color: '#fff', borderRadius: '50%', width: 16, height: 16, fontSize: 10,
                display: 'flex', alignItems: 'center', justifyContent: 'center', fontWeight: 700,
              }}>{alerts}</span>}
            </button>
            {/* Mute toggle */}
            <button onClick={() => { toggleMute(); setAlarmMuted(!alarmMuted); }} style={{
              ...btnStyle, borderColor: alarmMuted ? 'var(--warn)' : 'var(--border)',
              color: alarmMuted ? 'var(--warn)' : 'var(--fg)',
            }}>
              {alarmMuted ? (
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/><line x1="1" y1="1" x2="23" y2="23" stroke="currentColor" strokeWidth="2"/></svg>
              ) : (
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
              )}
            </button>
```

- [ ] **Step 5: Add AlarmPanel render at the end of the component**

Just before the closing `</div>` of the root element (the last closing div in the return), add:

```tsx
      {showAlarmPanel && <AlarmPanel onClose={() => setShowAlarmPanel(false)} />}
```

- [ ] **Step 6: Verify TypeScript compiles**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`
Expected: No errors.

- [ ] **Step 7: Commit**

```bash
git add client/src/App.tsx
git commit -m "feat(alarm): integrate alarm bell, mute, audio playback into App"
```

---

### Task 9: Add Web Push groundwork

**Files:**
- Modify: `src/storage.rs` (add `push_subscriptions` table)
- Modify: `src/models.rs` (add `PushSubscription` struct)
- Modify: `src/api/mod.rs` (add push stub routes)
- Create: `src/api/push.rs` (subscribe/unsubscribe stubs)
- Create: `client/public/sw.js` (service worker shell)
- Modify: `client/src/App.tsx` (notification permission on login)

- [ ] **Step 1: Add `PushSubscription` struct to `src/models.rs`**

Add after the `AlarmQuery` struct:

```rust
// ── Push subscription (groundwork for Web Push) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PushSubscription {
    pub id: i64,
    pub user: String,
    pub endpoint: String,
    pub p256dh: String,
    pub auth_key: String,
    pub created_at: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct PushSubscribeRequest {
    pub endpoint: String,
    pub p256dh: String,
    pub auth_key: String,
}
```

- [ ] **Step 2: Add `push_subscriptions` table to `init_schema` in `src/storage.rs`**

Add to the SQL in `init_schema`:

```sql
        CREATE TABLE IF NOT EXISTS push_subscriptions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            p256dh TEXT NOT NULL,
            auth_key TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
```

Add two storage methods to `impl Storage`:

```rust
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
```

Update the import in `storage.rs` to include `PushSubscription`:

```rust
use crate::models::{
    DataPoint, Device, ScreenLayoutRow, DataSource, Setting, TwinLayout, User, UserShortcut,
    Alarm, AlarmQuery, PushSubscription,
};
```

- [ ] **Step 3: Create `src/api/push.rs`**

```rust
use axum::extract::{Extension, State};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::models::*;
use super::{AppError, AppState, AuthUser};

/// POST /api/push/subscribe
pub async fn subscribe(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<PushSubscribeRequest>,
) -> Result<impl IntoResponse, AppError> {
    state
        .storage
        .save_push_subscription(&user.username, &body.endpoint, &body.p256dh, &body.auth_key)
        .await?;
    Ok(Json(json!({"ok": true})))
}

/// DELETE /api/push/subscribe
pub async fn unsubscribe(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let endpoint = body["endpoint"].as_str().unwrap_or("");
    state
        .storage
        .remove_push_subscription(&user.username, endpoint)
        .await?;
    Ok(Json(json!({"ok": true})))
}
```

- [ ] **Step 4: Register push module and routes in `src/api/mod.rs`**

Add to module declarations:

```rust
mod push;
```

Add to the protected router (after the alarm routes):

```rust
        .route(
            "/api/push/subscribe",
            post(push::subscribe).delete(push::unsubscribe),
        )
```

- [ ] **Step 5: Verify backend compiles**

Run: `cargo check 2>&1 | head -20`
Expected: No errors.

- [ ] **Step 6: Create `client/public/sw.js` (service worker shell)**

```js
// Service worker shell for Web Push Notifications (groundwork)
self.addEventListener('install', () => {
  console.log('[SW] Installed');
  self.skipWaiting();
});

self.addEventListener('activate', () => {
  console.log('[SW] Activated');
});

self.addEventListener('push', (event) => {
  console.log('[SW] Push received:', event.data?.text());
  // TODO: show notification using self.registration.showNotification()
});
```

- [ ] **Step 7: Add notification permission request in `client/src/App.tsx`**

Add a `useEffect` after the existing auth handling effects (after the `useEffect` that handles `authLoading`):

```ts
  // Request notification permission once on login
  useEffect(() => {
    if (!authed) return;
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission();
    }
  }, [authed]);
```

- [ ] **Step 8: Verify frontend compiles**

Run: `cd client && npx tsc --noEmit 2>&1 | head -20`
Expected: No errors.

- [ ] **Step 9: Commit**

```bash
git add src/models.rs src/storage.rs src/api/push.rs src/api/mod.rs client/public/sw.js client/src/App.tsx
git commit -m "feat(alarm): add Web Push groundwork (table, stubs, service worker, permission)"
```

---

### Task 10: Build and smoke test

- [ ] **Step 1: Build the Rust backend**

Run: `cargo build 2>&1 | tail -5`
Expected: `Finished` with no errors.

- [ ] **Step 2: Build the frontend**

Run: `cd client && npm run build 2>&1 | tail -10`
Expected: Build succeeds.

- [ ] **Step 3: Run the app and verify**

Start the server:
```bash
cargo run config.yaml
```

Open in browser:
1. Login with admin/admin
2. Verify the alarm bell icon appears in the header
3. Verify the mute button appears next to it
4. Click the bell — verify the alarm panel slides in from the right
5. Verify filter tabs (All / Active / Acknowledged) work
6. Verify no console errors

- [ ] **Step 4: Final commit (if any fixes needed)**

```bash
git add -A
git commit -m "fix(alarm): build fixes from smoke test"
```

---

## Self-Review Checklist

- [x] **Spec coverage:** Every section of the design spec has corresponding tasks
- [x] **Placeholder scan:** No TBDs, TODOs (except intentional push placeholder), no "implement later"
- [x] **Type consistency:** `Alarm` struct fields match between Rust (`models.rs`) and TypeScript (`types.ts`); API paths consistent between Rust routes and frontend fetch calls; storage method signatures match API handler calls
