# Voltrus Iced Client — Minimal Viewer Design

**Date**: 2026-04-28
**Scope**: Minimal native desktop viewer for SCADA terminals and operator workstations
**Status**: Approved

## Context

The GPUI native client was removed due to excessive dependency risk (1,237 transitive crates, experimental framework). A replacement is needed for scenarios where a browser is unavailable or undesirable (embedded terminals, air-gapped networks, operator stations). The web client remains the primary UI — this is a lightweight alternative.

## Scope

**Minimal viewer**: live sensor data + basic history charts. No 3D, no canvas drag, no SCADA 2D editor, no admin screens.

## Architecture

### Crate Structure

```
voltrus-iced/
├── Cargo.toml
└── src/
    ├── main.rs       # Entry point, window config, config loading
    ├── app.rs        # Iced App: new(), update(), view(), subscription()
    ├── theme.rs      # Dark theme constants, SCADA color palette
    ├── http.rs       # HTTP client wrapper (login, data, history, SSE)
    ├── models.rs     # Minimal types matching backend API responses
    └── views/
        ├── mod.rs
        ├── login.rs  # Login screen
        ├── grid.rs   # Live sensor grid with cards
        └── chart.rs  # History line chart (plotters-iced)
```

Standalone binary in repo root, not a workspace member. Communicates with backend exclusively via HTTP API.

### Dependencies

```toml
[dependencies]
iced = "=0.13.2"
iced_aw = { version = "=0.13.0", default-features = false, features = ["card"] }
plotters-iced = "=0.11.0"
plotters = "=0.3.7"
reqwest = { version = "=0.12.15", default-features = false, features = ["json", "rustls-tls"] }
tokio = { version = "=1.52.1", features = ["rt-multi-thread", "sync", "time"] }
serde = { version = "=1.0.228", features = ["derive"] }
serde_json = "=1.0.149"
anyhow = "=1.0.102"
time = "=0.3.47"
```

All deps pinned to exact versions for deterministic rebuilds. Shares reqwest/serde/tokio with backend.

### Data Flow

1. **Startup**: Load config from `iced-config.yaml` (server URL, optional credentials)
2. **Login**: `POST /api/auth/login` → session cookie stored in reqwest cookie jar
3. **Live data**: `GET /api/data` polled every 1s via `iced::time::every` subscription
4. **History**: `GET /api/history?range=30m&sensor=X` on demand when user selects a sensor
5. **Alarms**: Alarm status inferred from data values vs warn/crit thresholds

### State Machine

```
AppPhase::Login → AppPhase::Loading → AppPhase::Dashboard
                                      ↓
                              AppPhase::Error (with retry)
```

## Views

### Login Screen

- Two text inputs: username, password
- Login button
- Error message display (red text)
- Connection status indicator (server URL configurable)
- On success: transition to Loading phase

### Grid View (Dashboard)

- Sensor cards in a responsive grid (auto-flow, wrapping)
- Each card (iced_aw::Card) shows:
  - Sensor name
  - Current value with unit
  - Timestamp of last update
  - Color-coded border: green (normal), yellow (warning), red (critical)
- Top bar: connection status, screen selector dropdown, logout button
- Polling indicator (last update timestamp)

### Chart View (History)

- Triggered by clicking a sensor card
- Line chart using plotters-iced:
  - X axis: time
  - Y axis: value
  - Warning threshold line (yellow dashed)
  - Critical threshold line (red dashed)
- Range selector buttons: 30m, 1h, 6h, 24h
- Back button to return to grid
- Loading state while fetching history

## API Endpoints Used

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/auth/login` | POST | Authenticate, get session cookie |
| `/api/auth/logout` | POST | Clear session |
| `/api/data` | GET | Fetch current sensor values |
| `/api/data-sources` | GET | List data sources (names, units, thresholds) |
| `/api/history` | GET | Fetch historical data points |
| `/api/settings/branding` | GET | Get title/subtitle for window title |

## Configuration

`iced-config.yaml`:
```yaml
server_url: "http://localhost:3000"
poll_interval_secs: 1
default_history_range: "30m"
```

## Implementation Order

1. Scaffold `voltrus-iced/` with Cargo.toml and main.rs
2. Implement `http.rs` — login, data fetch, history fetch
3. Implement login view + auth flow
4. Implement grid view with live polling
5. Implement chart view with plotters-iced
6. Add config loading
7. Add alarm color coding
8. Test end-to-end against running backend

## Out of Scope

- 3D/Digital Twin view
- Canvas (drag/resize) view
- SCADA 2D editor
- Admin operations (user management, device CRUD)
- SSE streaming (polling is sufficient for minimal viewer)
- Multi-screen support (show all data sources)
- Push notifications

## Verification

1. `cd voltrus-iced && cargo build` — compiles
2. Start backend: `cargo run -- config.dev.yaml`
3. Start iced client: `cd voltrus-iced && cargo run`
4. Login with admin/admin
5. Verify sensor grid shows live data
6. Click a sensor → verify chart loads history
7. Verify alarm colors appear when thresholds exceeded
8. Verify logout works
