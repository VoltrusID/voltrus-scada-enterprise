# Voltrus Lite

**Industrial monitoring that doesn't get in your way.**

A lightweight, self-contained SCADA/IoT dashboard built with Rust and React. Designed for power meters, solar plants, and industrial equipment — no cloud dependency, no bloated middleware, no vendor lock-in.

Built by [JoyoDigitama](https://joyodigitama.com).

---

## Why Voltrus Lite?

Existing industrial monitoring tools are either too heavy ( requiring OPC UA, UNS, and three layers of translation ), too expensive ( cloud-gated SaaS ), or too rigid ( fixed dashboards you can't customize ). Voltrus Lite is the opposite — a single binary, a single config file, and a browser.

### It's Fast

- **Single-page app with SSE** — no page reloads, no loading spinners, no awkward white flashes between views. Live data streams in via Server-Sent Events the moment you open the dashboard.
- **Sub-100ms updates** — configurable polling intervals down to 1ms for Modbus registers. Turbo mode for when you need the fastest possible refresh.
- **Cold start in under a second** — the Rust binary boots, migrates the database, and starts serving before your coffee is ready.

### It's Lightweight

- **RAM footprint well under 100 MB** — typical idle usage measured in single-digit megabytes. The entire service + SQLite + SSE server runs lighter than a single Chrome tab.
- **Single binary deployment** — no runtime, no JVM, no Node.js. One compiled binary, one config YAML, done. Cross-compile for ARM Linux, x86_64, macOS, or wherever your plant runs.
- **Minimal dependency tree** — the entire backend uses ~20 carefully chosen Rust crates. No transitive dependency nightmares. This software will compile and run a decade from now.

### It's IT-Friendly

- **OIDC/SSO out of the box** — integrates with Keycloak, Auth0, Azure AD, or any OpenID Connect provider. Local authentication with Argon2 password hashing also included.
- **Role-based access control** — admin and user roles, managed through the UI.
- **Secure by default** — JWT cookie-based sessions, CORS configuration, no exposed secrets.

### It Speaks Modbus Natively

- **First-class Modbus TCP support** — because that's what power meters speak. No forcing Unified Name Spaces (UNS), OPC UA, MQTT bridges, or other translation layers that add latency and failure points.
- **Full register flexibility** — configurable unit ID, endianness (Big, Little, Swapped Big, Swapped Little), data types (Int16, Uint16, Int32, Float32), scaling multipliers, and custom polling intervals per register.
- **Automatic reconnection** — devices go offline and come back without manual intervention. Status is reflected in real-time.
- **Built-in Modbus simulator** — a separate `modbus-sim` binary for testing without physical hardware.

### It Has the Views You Need

| View | Description |
|------|-------------|
| **Grid** | Responsive card grid with drag-and-drop layout, sparkline charts, and live values. |
| **Canvas** | Free-form layout — drag sensors anywhere, resize freely. Think whiteboard meets dashboard. |
| **3D Digital Twin** | Three.js-powered 3D scene with solar panels, inverters, gateways, and custom equipment. Drag, rotate, connect equipment with lines. Camera positions saved per screen. |
| **2D SCADA** | Industrial diagram editor with transformers, circuit breakers, meters, switches, motors, batteries. Draw power connections and signal lines with waypoint routing. |

All views support **per-screen persistence** — layouts, camera positions, and equipment arrangements are saved independently for each screen.

### It Has Every Configuration You'd Want

- **Screens** — create unlimited screens (tabs) to organize sensors by area, equipment, or responsibility. Switch instantly, no reload.
- **Responsive breakpoints** — separate layouts for Desktop (24-col), Tablet (12-col), and Mobile (4-col). Edit each breakpoint independently in edit mode. No awkward layouts on phones or tablets.
- **Custom branding** — configurable title, subtitle, and logo. Make it yours.
- **Keyboard shortcuts** — fully customizable. Assign shortcuts to:
  - Screen switching (`Mod+1`, `Mod+2`, ...)
  - View switching (Grid, Canvas, 3D, 2D)
  - Mode switching (Live, History)
  - Sensor detail (jump directly to a sensor's detail chart)
  - Alarm panel toggle
- **Per-sensor configuration** — color, unit, min/max range, warning/critical thresholds, decimal precision, chart type, source type, equipment association.
- **Dashboard settings** — grid gap, refresh interval, history aggregation method, alarm audio mute.

### It Has Alarms

- **Threshold-based alerts** — configure warning (yellow) and critical (red) thresholds per sensor.
- **Audio notifications** — critical alarms trigger an audible alert with mute control.
- **Alarm panel** — real-time and historical alarm log with acknowledgment tracking (who acknowledged, when).
- **Cooldown periods** — prevent alarm spam from flapping values.

### It Has Smart Data Retention

Data storage is automatic and tiered — no manual archiving, no database administration:

| Tier | Resolution | Default Retention | Notes |
|------|-----------|-------------------|-------|
| Raw | Original polling interval | 7 days | Archived to `archive/YYYY-MM-DD.csv.gz` before deletion |
| Hourly | 1-hour aggregates | 90 days | Min, max, avg, count per hour |
| Daily | 1-day aggregates | Forever | Min, max, avg, count per day |

- **Automatic downsampling** — a background task runs every 6 hours (configurable), aggregates raw data older than 7 days into hourly buckets, then hourly data older than 90 days into daily buckets.
- **Transparent tier queries** — the history API automatically queries across all three tiers and merges the results. The frontend gets seamless data regardless of which tier it lives in.
- **CSV archive** — raw data is compressed and archived before deletion, so you never lose granularity if you need it later.
- **Configurable via `config.yaml`** — retention intervals, archive paths, and aggregation schedules are all adjustable. Works out of the box with sensible defaults.

### It Has Calculated Values

- **Expression engine** — define virtual sensors using mathematical formulas with register references:
  ```
  {Voltage_L1} * {Current_L1} * 1.732
  ```
- Supports arithmetic operators (`+`, `-`, `*`, `/`), parentheses, and unary minus.
- **Three source types**: Modbus (real device), Random Walk (synthetic test data), Expression (calculated).

### It Has History Done Right

- **Historical charting** — switch any view to History mode to see time-series data. Select from presets (30m, 1h, 6h, 1d, 7d, 30d) or custom ranges.
- **Aggregation options** — sampling or averaging for large time ranges to keep charts readable.
- **Full detail mode** — bypass aggregation to see every single data point.
- **Detail view per sensor** — click any sensor to see a dedicated detail chart with time range controls.
- **Multi-sensor overlay** — compare multiple sensors on the same detail chart.

### It Has System Monitoring

Built-in system resource panel showing:
- Host information and uptime
- CPU usage with visual gauge
- Memory usage (total, used, percentage)
- Disk usage (total, used, percentage)
- Process metrics — RSS memory, CPU%, uptime, and binary size of the Voltrus service itself

---

## Architecture at a Glance

```
┌─────────────────────────────────────────────┐
│  Browser (React SPA + Three.js)             │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────────┐   │
│  │ Grid │ │Canvas│ │ 3D   │ │ 2D SCADA │   │
│  └──┬───┘ └──┬───┘ └──┬───┘ └────┬─────┘   │
│     └────────┴────────┴──────────┘          │
│              SSE / REST API                  │
├─────────────────────────────────────────────┤
│  Rust Backend (Axum)                        │
│  ┌──────────┐ ┌────────┐ ┌───────────────┐  │
│  │ Modbus   │ │Auth    │ │ Retention      │  │
│  │ Poller   │ │(JWT+   │ │ (downsample,   │  │
│  │          │ │ OIDC)  │ │  archive)      │  │
│  └────┬─────┘ └────────┘ └───────────────┘  │
│       │              ┌───────────────┐       │
│       │              │ Expression    │       │
│       │              │ Engine        │       │
│       │              └───────────────┘       │
│       └──────────────┐                       │
│                      ▼                       │
│              ┌───────────────┐               │
│              │ SQLite        │               │
│              │ (auto-migrate)│               │
│              └───────────────┘               │
└─────────────────────────────────────────────┘
```

## Tech Stack

**Backend:**
- Rust (Axum, Tokio, tokio-modbus)
- SQLite (rusqlite with bundled SQLite)
- JWT + Argon2 + OIDC for authentication
- SSE for real-time data streaming
- ~20 total dependencies

**Frontend:**
- React + TypeScript
- Three.js for 3D digital twin
- react-grid-layout for responsive grid
- TanStack Query for data fetching
- Inline CSS (zero CSS framework dependency)

---

## Quick Start

```bash
# Build
cargo build --release

# Configure
cp config.example.yaml config.yaml
# Edit config.yaml with your Modbus device details

# Run
./target/release/voltrus-lite-rust-service

# Open
# http://localhost:3000
# Default login: admin / admin (change on first login)
```

## Configuration

Everything is configured through a single `config.yaml`:

```yaml
server:
  port: 3000
  cors_origins: []

database:
  path: data/voltrus.db

auth:
  jwt_secret: "change-me-in-production"
  token_expire_hours: 24
  oidc:                          # Optional — omit for local-only auth
    issuer: "https://keycloak.example/realms/myrealm"
    client_id: "voltrus-lite"
    client_secret: "secret"

retention:
  raw_days: 7
  hourly_days: 90
  daily_keep_forever: true
  archive_dir: archive
  check_interval_hours: 6

devices:
  - name: "Power Meter 1"
    address: "192.168.1.100:502"
    unit_id: 1
    interval_ms: 1000
    registers:
      - name: "Voltage_L1"
        address: 0
        type: Float32
        endian: Big
```

## What Voltrus Lite is Not

- Not a cloud platform — it runs on your machine, your network, your terms.
- Not an OPC UA gateway — it talks Modbus TCP directly because that's what your power meters speak.
- Not a historian replacement — but the tiered retention with CSV archiving covers most use cases.
- Not a 500-feature SCADA suite — it's the 20% that covers 80% of industrial monitoring needs, and it does those 20% extremely well.

## Roadmap

- **Native desktop client** (Iced/Rust) — the same dashboard experience without a browser, for environments where browsers aren't available or desired.
- **Push notifications** — browser push for alarm alerts when you're not watching the dashboard.
- **More Modbus device templates** — pre-built register maps for common power meters (Schneider, ABB, Janitza, etc.).
- **PDF/CSV export** — scheduled reports with chart snapshots.

---

## License

Proprietary — © JoyoDigitama. All rights reserved.
