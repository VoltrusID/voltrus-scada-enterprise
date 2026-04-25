# Alarm System Design

## Overview

Add a threshold-based alarm system to Voltrus Lite. When data source values cross their configured warn/crit thresholds, alarms trigger with looping audio playback and a persistent alarm list for acknowledgment. Backend records all alarms for history and future Web Push Notification integration.

## Architecture

**Hybrid approach:** Backend owns alarm state (creates records, persists to SQLite). Frontend detects breaches from existing `/api/data` responses for real-time audio/UI — no additional polling needed.

Note: This design uses the upcoming `data_sources` naming (the `sensors` → `data_sources` rename is being done in parallel).

## Section 1: Data Model & Backend

### New `alarms` table (SQLite)

```sql
CREATE TABLE IF NOT EXISTS alarms (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    source_key      TEXT NOT NULL,
    severity        TEXT NOT NULL CHECK(severity IN ('warning', 'critical')),
    value           REAL NOT NULL,
    threshold       REAL NOT NULL,
    message         TEXT,
    triggered_at    INTEGER NOT NULL,
    resolved_at     INTEGER,
    acknowledged_at INTEGER,
    acknowledged_by TEXT
);
CREATE INDEX idx_alarms_active ON alarms(source_key, severity, acknowledged_at);
CREATE INDEX idx_alarms_triggered ON alarms(triggered_at DESC);
```

### Backend alarm engine

Runs inside the existing Modbus poller / data generation loop (every ~1s):

1. After caching latest values, iterate all data sources
2. For each source, compare current value against its `warn_val` and `crit_val`
3. If value >= `crit_val`, severity = "critical"; else if value >= `warn_val`, severity = "warning"
4. Check if an active (unacknowledged, unresolved) alarm already exists for this `source_key` + `severity`
5. If no active alarm exists, INSERT a new alarm record
6. If value has returned to normal and an active alarm exists, set `resolved_at = now`

### New API endpoints

All protected (require cookie auth):

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/alarms?status=active\|all&limit=50` | Fetch alarm list |
| POST | `/api/alarms/{id}/acknowledge` | Acknowledge single alarm |
| POST | `/api/alarms/acknowledge-all` | Bulk acknowledge all active alarms |

**GET /api/alarms response:**
```json
[
  {
    "id": 1,
    "source_key": "plts_1_voltage",
    "source_name": "PLTS 1 Voltage",
    "severity": "critical",
    "value": 245.3,
    "threshold": 240.0,
    "message": "Voltage reached 245.3V (critical: 240V)",
    "triggered_at": 1714032000,
    "resolved_at": null,
    "acknowledged_at": null,
    "acknowledged_by": null
  }
]
```

**POST /api/alarms/{id}/acknowledge** — sets `acknowledged_at` and `acknowledged_by` (current user).

**POST /api/alarms/acknowledge-all** — acknowledges all unacknowledged alarms for current user.

## Section 2: Frontend — Audio & Real-time Detection

### Audio system

- Audio file (user-provided, e.g. `alarm.mp3`) placed in `client/public/`
- Uses Web Audio API (`AudioContext`) with HTML5 `<audio>` element, `loop: true`
- Single sound for both warning and critical alarms
- Playback rules:
  - Starts when frontend detects a new threshold breach
  - Stops when all breaches are resolved (values normal) OR all active alarms are acknowledged
- Mute button in header bar (session-scoped, silences audio without acknowledging)

### Frontend alarm detection

In the existing `requestAnimationFrame` loop where `currentValues` are updated:

1. Check each incoming value against its `warn` and `crit` thresholds
2. Maintain a `Set<string>` of currently-breached source keys with their severity
3. On new breach (key enters set): start audio playback if not muted
4. On normalization (key leaves set): check remaining breaches, stop audio if set is empty
5. Update alarm badge count from set size (no extra fetch needed)

### Header bar indicator

- Bell icon with numeric badge showing count of active breaches
- Pulses/glows (CSS animation) when audio is actively playing
- Click opens the alarm side panel
- Mute toggle button next to bell (speaker icon with slash when muted)

## Section 3: Alarm List Side Panel

### Panel layout

- Slides in from right edge, 360px wide, full viewport height
- Overlay with semi-transparent backdrop (click backdrop to close)
- Dark theme matching existing dashboard: `background: var(--card)`, borders `var(--border)`

### Panel header

- Title: "Alarms"
- "Acknowledge All" button (styled like existing btnStyle)
- Close button (X)

### Filter tabs

- Three tabs: All / Active / Acknowledged
- Styled like existing screen navigation tabs (`tabStyle`)

### Alarm entries (sorted newest first)

Each entry shows:
- Severity badge: WARNING (yellow `var(--warn)`) or CRITICAL (red `var(--crit)`)
- Source name (resolved from key via sensor list)
- Trigger info: `"245.3V (crit: 240V)"`
- Triggered timestamp (formatted with `fmtTs`)
- Status: active (pulsing red/yellow dot) / acknowledged (checkmark, dimmed) / resolved (gray, dimmed)
- "Acknowledge" button per alarm (only for active/unacknowledged)

### Panel behavior

- Fetches `/api/alarms?status=active&limit=50` when opened
- After acknowledge action, refetches to update the list
- Filter changes trigger a new fetch with updated `status` param
- Badge count on bell icon updates from real-time detection (no extra fetch)

## Section 4: Web Push Notification Preparation

### Backend groundwork

New `push_subscriptions` table:
```sql
CREATE TABLE IF NOT EXISTS push_subscriptions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user        TEXT NOT NULL,
    endpoint    TEXT NOT NULL,
    p256dh      TEXT NOT NULL,
    auth_key    TEXT NOT NULL,
    created_at  INTEGER NOT NULL
);
```

Stub API endpoints (protected):
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/push/subscribe` | Store push subscription |
| DELETE | `/api/push/subscribe` | Remove subscription |

In the alarm engine, after creating a new alarm:
```rust
// TODO: send web push notification to all push_subscriptions
```

### Frontend groundwork

- Service worker shell: `client/public/sw.js` — logs push events, no real handling yet
- On login, a `useEffect` checks `Notification.permission` and requests if not yet decided (one-time, non-nagging)
- In DashboardSettings, a "Notification Settings" section with a disabled toggle labeled "Push Notifications (Coming Soon)"

No actual push delivery — just the subscription flow and data model ready for a future PR.

## Implementation Order

1. Backend: alarms table, alarm engine in poller, API endpoints
2. Frontend: alarm detection from data responses, audio playback system
3. Frontend: alarm side panel UI with acknowledge flow
4. Backend + Frontend: Web Push groundwork (tables, stubs, service worker shell)
