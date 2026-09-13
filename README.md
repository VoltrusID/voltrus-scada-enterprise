# Voltrus

**Industrial monitoring that doesn't get in your way.**

A lightweight, self-contained SCADA/IoT platform built with Rust and React. Single binary, multi-protocol, zero cloud dependency.

Built by [JoyoDigitama](https://joyodigitama.com).

---

## Features

### Multi-Protocol Polling

| Protocol | Feature Flag | Default |
|----------|-------------|---------|
| Modbus TCP | `modbus` | Yes |
| OPC-UA | `opcua` | Yes |
| Siemens S7 | `s7` | Yes |
| BACnet/IP | `bacnet` | Yes |
| DNP3 | `dnp3` | Yes |
| EtherNet/IP (CIP) | `ethernet-ip` | Yes |
| MQTT Broker (embedded) | `mqtt-broker` | Yes |
| SNMP Network Scan | `snmp_scan` | Yes |

### Dashboard Views

| View | Description |
|------|-------------|
| **Grid** | Responsive card grid with drag-and-drop layout and sparkline charts |
| **Canvas** | Free-form layout — drag sensors anywhere, resize freely |
| **3D Digital Twin** | Three.js-powered 3D scene with solar panels, inverters, and custom equipment |
| **2D SCADA** | Industrial diagram editor with transformers, breakers, meters, switches, and motors |

All views support per-screen persistence with independent responsive breakpoints (Desktop/Tablet/Mobile).

### Flow Engine

Visual automation builder — chain together filtering, threshold checks, script execution, MQTT publish, and notification nodes. Design automations in the browser, run them on the edge.

### Script Engine

Write JavaScript directly in the browser editor, test with sample input, and deploy to the flow engine. Powered by [Boa](https://boajs.dev/) (pure Rust, no C dependencies). Sandboxed with runtime limits and security restrictions.

### Alarms & Notifications

- Threshold-based alerts (warning/critical) with configurable cooldown
- Audio notifications with mute control
- Alarm panel with acknowledgment tracking
- Email notifications (SMTP)
- Shift handover logging

### Data Retention

| Tier | Resolution | Default Retention |
|------|-----------|-------------------|
| Raw | Original polling interval | 7 days |
| Hourly | 1-hour aggregates | 90 days |
| Daily | 1-day aggregates | Forever |

Automatic downsampling runs in the background. Raw data archived to `archive/YYYY-MM-DD.csv.gz` before deletion.

### Security

- JWT cookie-based sessions with Argon2 password hashing
- OIDC/SSO integration (Keycloak, Auth0, Azure AD)
- LDAP and SAML authentication
- Role-based access control (admin/user)
- Rate limiting, CSRF protection, intrusion detection
- Audit logging

### Other Features

- **Tag browser** — organize and browse all data points across devices
- **Asset inventory** — OT asset management with hierarchy and metadata
- **Config snapshots** — point-in-time config backup and restore
- **Data export** — CSV/JSON export with time range selection
- **Data import** — CSV import wizard, Node-RED flow import
- **Keyboard shortcuts** — fully customizable per-user
- **Custom branding** — title, subtitle, logo
- **Multi-language** — i18n with locale files
- **License tiers** — Trial, Starter, Professional, Enterprise with ed25519 signing

---

## Architecture

```
┌─────────────────────────────────────────────┐
│  Browser (React 19 SPA + Three.js)          │
│  Grid │ Canvas │ 3D Twin │ 2D SCADA         │
│         SSE / REST / WebSocket               │
├─────────────────────────────────────────────┤
│  Rust Backend (Axum + Tokio)                │
│  ┌──────────┐ ┌────────┐ ┌───────────────┐  │
│  │ Protocol │ │ Auth   │ │ Flow Engine   │  │
│  │ Pollers  │ │(JWT+   │ │ + Scripts     │  │
│  │(9 protos)│ │ OIDC)  │ │ (boa_engine)  │  │
│  └────┬─────┘ └────────┘ └───────────────┘  │
│       ▼              ┌───────────────┐       │
│  ┌─────────┐         │ Retention      │       │
│  │ Device  │         │ (downsample,   │       │
│  │ Cache   │         │  archive)      │       │
│  └────┬────┘         └───────────────┘       │
│       ▼                                     │
│  ┌───────────────┐                          │
│  │ SQLite        │                          │
│  │ (auto-migrate)│                          │
│  └───────────────┘                          │
└─────────────────────────────────────────────┘
```

---

## Quick Start

```bash
# Build (includes all protocols + scripts)
cargo build --release

# Build without scripts (saves ~15 MB)
cargo build --release --no-default-features --features "snmp_scan,modbus,opcua,mqtt-broker,bacnet,dnp3,ethernet-ip,s7"

# Configure
cp config.yaml config.yaml
# Edit config.yaml with your device details

# Run
./target/release/voltrus config.yaml

# Open http://localhost:3000
# Default login: admin / admin (change on first login)
```

### Cross-Compilation & Release

```bash
# Requires: zig, cargo-zigbuild, bun, wrangler

# Single target
rustup target add x86_64-unknown-linux-musl
bash -c 'ulimit -n 65536 && cargo zigbuild --release --target x86_64-unknown-linux-musl'

# Full release (all platforms + R2 upload)
./scripts/release-local.sh

# Options
./scripts/release-local.sh --target aarch64       # macOS ARM64 only
./scripts/release-local.sh --target linux-x86_64   # Linux x86_64 only
./scripts/release-local.sh --skip-upload          # Build only, skip R2 upload
NOTARIZE=1 ./scripts/release-local.sh              # With macOS notarization
```

### Deploy to VPS

```bash
# Build + deploy in one step
./deploy-vps.sh

# Options
./deploy-vps.sh --build-only     # build only
./deploy-vps.sh --deploy-only    # deploy existing build
./deploy-vps.sh --frontend-only  # rebuild frontend only
```

### Frontend Development

```bash
cd client
bun install
bun run dev        # dev server at localhost:5173, proxies API to :3100
bun run build      # production build → ../static/
```

---

## Configuration

Everything is configured through a single `config.yaml`:

```yaml
server:
  port: 3000
  cors_origins: []

database:
  path: data.db

auth:
  jwt_secret: "change-me-in-production"
  token_expire_hours: 24
  oidc:                          # Optional — omit for local-only auth
    issuer: "https://keycloak.example/realms/myrealm"
    client_id: "voltrus"
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

---

## Tech Stack

**Backend:** Rust (Axum, Tokio, rusqlite) — single binary, no runtime

**Frontend:** React 19 + TypeScript + Vite 8 (Rolldown) + Three.js — inline CSS, zero framework dependency

**License Server:** Cloudflare Worker + D1

**Landing Page:** Static HTML on Cloudflare Pages

---

## Project Structure

```
voltrus/                          # Core application (this repo)
├── src/                          # Rust backend
│   ├── main.rs                   # Entry point
│   ├── api/                      # ~94 route handlers
│   ├── storage/                  # SQLite persistence (~40 modules)
│   ├── flow_engine/              # Visual automation engine
│   ├── script_engine.rs          # JS sandbox (boa_engine)
│   ├── auth/                     # Local, OIDC, LDAP, SAML
│   ├── security/                 # Rate limiting, CSRF, audit
│   └── bin/                      # Simulator binaries
├── client/                       # React frontend
│   ├── src/
│   │   ├── App.tsx               # Root component
│   │   ├── queries/              # React Query hooks (~55 files)
│   │   ├── api/                  # WebSocket client, commands
│   │   ├── components/           # Reusable UI components
│   │   ├── views/                # Page-level views
│   │   └── twin/                 # 3D digital twin (Three.js)
│   └── vite.config.ts
├── scenarios/                    # Integration test scenarios
├── sdks/                         # TS + Python client SDKs (source of truth) — published to github.com/VoltrusID/voltrus-gateway, see AGENTS.md
├── deploy/                       # Systemd service file
├── scripts/
│   └── release-local.sh          # Cross-compile + R2 upload
├── deploy-vps.sh                 # Build + deploy to VPS
└── Cargo.toml
```

---

## License

Proprietary — © JoyoDigitama. All rights reserved.
