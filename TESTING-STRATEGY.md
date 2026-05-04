# Voltrus Testing Strategy

> Pre-production testing guide for Voltrus SCADA v0.34.
> Covers simulator setup, manual click-testing, API smoke tests, and automated test execution.

---

## Table of Contents

1. [Quick Start — Launch a Study Case](#1-quick-start--launch-a-study-case)
2. [Simulator Reference](#2-simulator-reference)
3. [Study Case: Solar Farm — 100kW Rooftop PV Plant](#3-study-case-solar-farm--100kw-rooftop-pv-plant)
4. [Study Case: Water Treatment — Municipal 50,000 m³/d](#4-study-case-water-treatment--municipal-50000-m³d)
5. [Study Case: Factory Floor — Food & Beverage Filling Line](#5-study-case-factory-floor--food--beverage-filling-line)
6. [Study Case: Data Center — Tier-III 2MW](#6-study-case-data-center--tier-iii-2mw)
7. [Study Case: Oil & Gas — Remote Well Pad](#7-study-case-oil--gas--remote-well-pad)
8. [Manual Feature Test Matrix](#8-manual-feature-test-matrix)
9. [API Smoke Tests](#9-api-smoke-tests)
10. [Automated Test Suite](#10-automated-test-suite)
11. [CLI Command Tests](#11-cli-command-tests)
12. [Protocol-Specific Deep Tests](#12-protocol-specific-deep-tests)
13. [Performance & Load Testing](#13-performance--load-testing)
14. [Disaster Recovery Tests](#14-disaster-recovery-tests)
15. [Production Readiness Checklist](#15-production-readiness-checklist)
16. [Real Device References](#16-real-device-references)

---

## 1. Quick Start — Launch a Study Case

Each study case runs **multiple real devices** across different protocols simultaneously.
A single command starts all simulators + Voltrus in a tmux session.

### Build

```bash
cd voltrus/
cargo build --release
```

### Launch a scenario

```bash
# Solar farm (SMA inverter + PM5560 meter + S7 weather + MQTT)
./run-scenario.sh solar-farm

# Water treatment (S7 PLC + E+H flow + ABB analyzer + BACnet + MQTT)
./run-scenario.sh water-treatment

# Factory floor (CompactLogix + PM5560 + S7 CIP + DNP3 genset)
./run-scenario.sh factory-floor

# Data center (Raritan PDU + PM5560 UPS + Trane CRAC + MQTT env + OPC-UA)
./run-scenario.sh data-center

# Oil & gas well pad (Daniel flow + SEL relay + ROC800 + S7 compressor + MQTT)
./run-scenario.sh oil-gas-well
```

### Verify

1. Open http://localhost:3000 in browser
2. Copy admin password from terminal output
3. Login → change password when prompted
4. You should see screens with live updating values from all devices

**✅ If you see live data in the dashboard, the full stack works.**

### Stop

```bash
tmux kill-session -t voltrus-solar-farm    # replace with your scenario name
```

### Run individual simulators

```bash
# List all device profiles and register maps
cargo run --bin modbus-sim -- --list

# Run specific device on specific port
cargo run --bin modbus-sim -- --scenario solar-farm --role inverter --port 5020
cargo run --bin modbus-sim -- --scenario oil-gas-well --role gas-flow --port 5020

# With fault injection (10% delay, 2% disconnect, 5% garbage)
cargo run --bin modbus-sim -- --scenario water-treatment --role flowmeter \
    --port 5020 --delay 0.10 --disconnect 0.02 --garbage 0.05
```

---

## 2. Simulator Reference

Each simulator mimics a real PLC/device. All support fault injection for resilience testing.
The Modbus simulator supports `--scenario` and `--role` flags to select from 6 real device profiles.

### Binary Reference

| Simulator | Binary | Default Port | Protocol | Simulated Device | Fault Flags |
|-----------|--------|-------------|----------|------------------|-------------|
| Modbus TCP | `modbus-sim` | `5020` | Modbus TCP | Selectable via `--scenario --role` | `--delay`, `--disconnect`, `--garbage` |
| S7 | `s7-sim` | `10200` | S7 Communication | Siemens S7-1200/1500 | `--delay`, `--disconnect`, `--garbage` |
| OPC-UA | `opcua-sim` | `4840` | OPC-UA (None/Basic256Sha256) | Process variables | `--delay`, `--disconnect` |
| EtherNet/IP | `enip-sim` | `44818` | CIP over ENIP | Allen-Bradley CompactLogix | `--delay`, `--disconnect`, `--garbage` |
| DNP3 | `dnp3-sim` | `20000` | DNP3 outstation | SEL-351S / ROC800 | — |
| BACnet/IP | `bacnet-sim` | `47809` | BACnet/IP UDP | Trane Tracer SC+ / JC FEC | — |
| MQTT | `mqtt-sim` | `1883` | MQTT 3.1.1 broker | Scenario-specific topics | `--delay`, `--disconnect`, `--garbage` |

### Device Profiles (Modbus `--role` flag)

Run `cargo run --bin modbus-sim -- --list` to see all profiles with full register maps.

| Role | Manufacturer | Model | Description | Registers |
|------|-------------|-------|-------------|----------|
| `inverter` | SMA | STP 60-US-10 | Sunny Tripower 60kW solar inverter | 20 (DC/AC, yield, irradiance) |
| `grid-meter` | Schneider | PM5560 | 3-phase power quality meter (0.2S) | 16 (power, energy, THD) |
| `flowmeter` | Endress+Hauser | Promag 10W DN50 | Electromagnetic flowmeter | 7 (flow, velocity, temp) |
| `analyzer` | ABB | AZL402 | Water quality analyzer | 7 (pH, ORP, turbidity, DO, Cl₂) |
| `gas-flow` | Emerson | Daniel 1000 | Gas flow computer (AGA3/AGA8) | 11 (ΔP, flow, gas chromatograph) |
| `pdu` | Raritan | PX3-5902V | Intelligent rack PDU 3-phase 32A | 12 (per-phase, temp, humidity) |

### Fault injection

All fault flags take a probability `0.0`–`1.0`:

```bash
# 10% delay, 2% disconnect, 5% garbage
cargo run --bin modbus-sim -- --scenario solar-farm --role inverter \
    --port 5020 --delay 0.10 --disconnect 0.02 --garbage 0.05
```

| Flag | Effect | Use Case |
|------|--------|----------|
| `--delay 0.1` | 10% of requests get 500ms delay | Test timeout handling |
| `--disconnect 0.02` | 2% chance of connection drop | Test reconnection logic |
| `--garbage 0.05` | 5% chance of corrupt response | Test data validation |

### Physical realism

All profiles model real device behavior:

- **Load profiles**: sinusoidal ±15% variation over 15-minute cycles
- **Voltage sag**: heavier load → slight voltage drop (realistic transformer regulation)
- **THD**: current THD inversely proportional to load (VFD behavior)
- **Energy counters**: u64 monotonically increasing, accumulated from instantaneous power
- **Temperature**: self-heating proportional to current burden
- **All ranges**: match real device datasheets (see Section 16 for references)

---

## 3. Study Case: Solar Farm — 100kW Rooftop PV Plant

```
                     ┌──────────────┐
    ☀ Irradiance ──→ │              │
                     │  SMA STP 60  │──→ DC: 580V / 101A
    PV Strings ────→ │  Inverter    │──→ AC: 230V / 86A / 56.5kW
                     │  (Modbus)    │──→ Yield: 452 MWh lifetime
                     └──────┬───────┘
                            │ AC output
                     ┌──────▼───────┐
                     │  PM5560      │──→ Import: 1,587 MWh
                     │  Grid Meter  │──→ Export: 2.5 MWh
                     │  (Modbus)    │──→ PF: 0.96 / 50Hz
                     └──────┬───────┘
                            │
                     ┌──────▼───────┐
                     │  Grid        │  400V 3-phase TN-S
                     └──────────────┘
```

| Device | Protocol | Port | Role | Manufacturer | Model |
|--------|----------|------|------|-------------|-------|
| Solar inverter | Modbus TCP | 5020 | `inverter` | SMA | STP 60-US-10 |
| Grid meter | Modbus TCP | 5021 | `grid-meter` | Schneider | PM5560 |
| Weather station | S7 Comm | 10200 | `solar-weather` | Siemens | S7-1200 + Kipp&Zonen |
| Weather gateway | MQTT | 1883 | `solar-weather` | — | MQTT gateway |

**Start:** `./run-scenario.sh solar-farm`

**What to test:**
- Inverter DC/AC efficiency tracking
- Grid import/export energy balance
- Irradiance vs power output correlation
- Daily yield accumulation
- Alarm: low irradiance, inverter overtemperature, grid frequency deviation

---

## 4. Study Case: Water Treatment — Municipal 50,000 m³/d

```
    Raw Water ──→ [Coagulation] ──→ [Filtration] ──→ [Disinfection] ──→ Distribution
         │              │                 │                  │
    E+H Flow       S7-1200 DB1       S7-1200 DB2        S7-1200 DB3
    (Modbus)       Coagulant dose    Filter headloss    Chlorine residual
    250 m³/h       pH 7.2            Backwash cycle     Clearwell level
                   Turbidity 15 NTU  3 filters in par.  CT value 45 mg·min/L
         │              │                 │                  │
    ABB AZL402 ──── Water quality: pH, ORP, turbidity, DO, conductivity ────┐
    (Modbus)                                                                 │
                                                                             │
    PM5560 ── Plant power: 95kW, PF 0.96                                    │
    (Modbus)                                                                 │
                                                                             │
    JC HVAC ── Building climate: zones, AHU, CO2 ───────────────────────────┘
    (BACnet)

    MQTT ── SCADA telemetry: pumps, tanks, alarms
```

| Device | Protocol | Port | Manufacturer | Model |
|--------|----------|------|-------------|-------|
| Process PLC | S7 Comm | 10200 | Siemens | S7-1200 (3 DBs) |
| Raw water flow | Modbus TCP | 5020 | Endress+Hauser | Promag 10W DN50 |
| Water quality | Modbus TCP | 5021 | ABB | AZL402 |
| Power meter | Modbus TCP | 5022 | Schneider | PM5560 |
| Building HVAC | BACnet/IP | 47809 | Johnson Controls | Facility Explorer |
| SCADA gateway | MQTT | 1883 | — | WTP-SCADA-GW |

**Start:** `./run-scenario.sh water-treatment`

**What to test:**
- Process variable trending (pH, turbidity, chlorine)
- S7 multi-DB read (coagulation DB1 → filtration DB2 → disinfection DB3)
- Filter headloss accumulation and backwash trigger
- BACnet HVAC zone temperature control
- MQTT alarm telemetry (turbidity high, chlorine low)
- OEE: plant throughput vs capacity (50,000 m³/d design)

---

## 5. Study Case: Factory Floor — Food & Beverage Filling Line

```
    ┌─────────────────────────────────────────────────┐
    │              CompactLogix L33ER                   │
    │              (EtherNet/IP :44818)                  │
    │                                                    │
    │  Line Speed: 450 bpm    Fill Temp: 4.5°C          │
    │  Filler Level: 78%     CO2 Press: 3.2 bar         │
    │  Labeler: 448 bpm      Case Count: 2,847          │
    └────────────────┬────────────────────────────────┘
                     │
    ┌────────────────┼────────────────────────────────┐
    │                │                                  │
    ▼                ▼                                  ▼
    PM5560          S7-1200 CIP                       SEL-351S
    Main power      Clean-in-Place                    Backup genset
    95kW PF 0.96    Caustic 72°C 2.5%                 DNP3 relay
    400V 3ph        Acid 65°C 1.2%                    Motor protection
```

| Device | Protocol | Port | Manufacturer | Model |
|--------|----------|------|-------------|-------|
| Line PLC | EtherNet/IP | 44818 | Allen-Bradley | CompactLogix L33ER |
| Main power | Modbus TCP | 5020 | Schneider | PM5560 |
| CIP system | S7 Comm | 10200 | Siemens | S7-1200 |
| Genset relay | DNP3 | 20000 | SEL | SEL-351S |

**Start:** `./run-scenario.sh factory-floor`

**What to test:**
- ENIP tag browsing (Program:MainProgram)
- Production line speed tracking (bottles/min)
- CIP cycle phases (rinse → caustic → rinse → acid → sanitize)
- Power demand during CIP heating (spike detection)
- DNP3 outstation communication with relay

---

## 6. Study Case: Data Center — Tier-III 2MW

```
    Utility ──→ UPS ──→ PDU Row 1 ──→ Server Racks
                    │                      │
                    │    ┌───── CRAC ──────┤
                    │    │  Supply: 15°C   │
                    │    │  Return: 28°C   │
                    │    │  Fan: 78%       │
                    │    └─────────────────┘
                    │
                    └──→ PDU Row 2 ──→ Server Racks
                                       │
                            ┌── MQTT Sensors ──┤
                            │  Cold aisle: 21°C │
                            │  Hot aisle: 32°C  │
                            │  Humidity: 45%    │
                            │  Leak: OK         │
                            │  Door: Closed     │
                            └───────────────────┘
                                       │
                            ┌── OPC-UA DCIM ────┤
                            │  Aggregated view   │
                            └────────────────────┘
```

| Device | Protocol | Port | Manufacturer | Model |
|--------|----------|------|-------------|-------|
| Rack PDU | Modbus TCP | 5020 | Raritan | PX3-5902V |
| UPS meter | Modbus TCP | 5021 | Schneider | PM5560 |
| CRAC unit | BACnet/IP | 47809 | Trane | Tracer SC+ |
| Env sensors | MQTT | 1883 | — | DC-ENV-GW-01 |
| DCIM server | OPC-UA | 4840 | — | Aggregation server |

**Start:** `./run-scenario.sh data-center`

**What to test:**
- Per-outlet PDU power monitoring
- UPS load percentage and battery runtime
- BACnet CRAC supply/return temperature differential
- MQTT environmental alerts (hot aisle >35°C, leak detection, door open)
- OPC-UA DCIM aggregation of all data center metrics
- PUE calculation (Total facility power / IT equipment power)

---

## 7. Study Case: Oil & Gas — Remote Well Pad

```
    Wellhead ──→ Compressor ──→ Meter Run ──→ Pipeline
       │             │              │
    Tubing:      Suction:       Diff ΔP: 85" H2O
    1850 psig    745 psig       Static: 745 psig
    Casing:      Discharge:     Flow: 1250 MSCFH
    320 psig     1200 psig      Gas: 88.5% CH₄
                   │            H₂S: 12 ppm
                   │
              ┌────┴────┐
              │         │
           SEL-351S   ROC800
           Motor      RTU Flow
           Protect    Aggregator
           (DNP3)     (DNP3)
              │         │
              └────┬────┘
                   │
              Satellite ──→ MQTT ──→ Control Room
              Gateway       620ms latency
```

| Device | Protocol | Port | Manufacturer | Model |
|--------|----------|------|-------------|-------|
| Flow computer | Modbus TCP | 5020 | Emerson/Daniel | Daniel 1000 |
| Motor relay | DNP3 | 20000 | SEL | SEL-351S |
| Flow RTU | DNP3 | 20001 | Emerson | ROC800 |
| Compressor | S7 Comm | 10200 | Siemens | S7-1200 |
| Satellite GW | MQTT | 1883 | — | WELL-SAT-GW-01 |

**Start:** `./run-scenario.sh oil-gas-well`

**What to test:**
- Gas flow computation (AGA3 orifice: ΔP, P, T → flow rate)
- Gas chromatograph composition (CH₄, CO₂, H₂S)
- Compressor vibration monitoring and oil temperature
- DNP3 dual-outstation polling (SEL relay + ROC RTU)
- MQTT satellite link quality and latency monitoring
- Well pad security (fence breach alarm)

---

## 8. Manual Feature Test Matrix

Test each feature by clicking through the UI. Mark each ✅ or ❌.

### 13.1 Authentication & User Management

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| A1 | First login | Open `http://localhost:3000`, paste admin password from terminal | Dashboard loads |
| A2 | Forced password change | Login with default password | Prompt to set new password (min 8 chars) |
| A3 | Logout | Click user avatar → Log Out | Returns to login screen |
| A4 | Re-login | Enter new credentials | Dashboard loads |
| A5 | Wrong password | Enter wrong password | "Login failed" error message |
| A6 | Create user | Settings → Users → Add User → fill username, password, role | User appears in list |
| A7 | Role assignment | Create user with "viewer" role → login as that user | Limited UI (no admin panels) |
| A8 | Password reset CLI | `voltrus --reset-admin-password test-config.yaml` | New password printed, old password invalidated |
| A9 | CSRF token | Make cross-origin request | Blocked by CSRF protection |
| A10 | Session expiry | Wait for token expiry (or shorten JWT TTL) | Redirected to login |

### 13.2 Dashboard — Live View

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| D1 | Screen tabs | Click "PLTS 1" and "PLTS 2" tabs | Different data sources per screen |
| D2 | Card view | Click "Grids" button | Sensor cards with sparklines |
| D3 | Canvas view | Click "Canvas" button | Draggable widget layout |
| D4 | 3D digital twin | Click "3D" button | Three.js scene loads with equipment models |
| D5 | SCADA 2D view | Click "2D" button | SVG-based SCADA canvas with symbols |
| D6 | Create screen | Click "+" tab → enter name | New empty screen appears |
| D7 | Rename screen | Right-click tab → Rename → type new name | Tab updates |
| D8 | Delete screen | Right-click tab → Delete → confirm | Screen removed |
| D9 | Add data source | Click "+" on a screen → select tag | Card appears with live data |
| D10 | Value updating | Watch a card for 10s | Value changes every ~1s (polling interval) |
| D11 | Status indicator | Observe card dot colors | Green = good, Yellow = warning, Red = critical |
| D12 | Colorblind mode | Settings → toggle colorblind indicators | Status dots change to shapes (✓ △ ✕) |

### 13.3 Sensor Detail & History

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| H1 | Open detail | Click any sensor card | Detail panel opens with larger chart |
| H2 | Time ranges | Click 1h / 6h / 24h / 7d / 30d buttons | Chart updates to selected range |
| H3 | Custom range | Click calendar icon → pick dates | Chart shows exact range |
| H4 | Full detail | Toggle "Full Detail" switch | Raw data points visible (no aggregation) |
| H5 | Auto aggregate | Toggle "Auto Aggregate" switch | Chart uses appropriate aggregation |
| H6 | Turbo mode | Click lightning bolt | Faster rendering for large datasets |
| H7 | Data export | Click Export → CSV | CSV downloads with timestamp, value, quality |
| H8 | Annotations | Click data point → "Add Annotation" → type text | Annotation pin appears on chart |
| H9 | Edit annotation | Click annotation → edit text → save | Text updates |
| H10 | Multi-sensor | Add multiple sensors to detail view | All series overlay on chart |

### 13.4 Alarms

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| AL1 | Trigger alarm | Stop modbus-sim → wait → restart | Alarm fires when device goes offline |
| AL2 | Threshold alarm | Set warn < 100V → restart simulator with low values | Yellow alarm appears |
| AL3 | Critical alarm | Set crit threshold → exceed it | Red alarm appears |
| AL4 | Alarm panel | Click bell icon in header | Alarm panel opens with list |
| AL5 | Acknowledge one | Click "Ack" on an alarm | Alarm moves to "Acknowledged" state |
| AL6 | Acknowledge all | Click "Acknowledge All" | All active alarms acknowledged |
| AL7 | Shelf alarm | Click "Shelf" → set duration (5 min) | Alarm suppressed, auto-restores after 5 min |
| AL8 | Unshelf alarm | Click "Unshelf" | Alarm becomes active again |
| AL9 | Alarm summary | Observe summary counts | Active/acknowledged/shelved counts correct |
| AL10 | Non-repeating | Alarm stays active → don't ack | Only ONE alarm event, not repeated |
| AL11 | Cooldown | Set alarm cooldown (60s) → value oscillates | No alarm flood during cooldown |
| AL12 | Real-time updates | Keep alarm panel open → trigger alarm | Alarm appears without page refresh (SSE) |

### 13.5 Commands & Writes

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| C1 | Write value | Click sensor card → Write → enter value → submit | `202 Accepted` with command_id |
| C2 | Command tracking | After write → check Commands panel | Shows pending → sent → acked status progression |
| C3 | Cancel command | Write → immediately click "Cancel" | Status changes to "cancelled" |
| C4 | Write policy | Settings → enable write protection per device | Writes blocked until policy allows |
| C5 | Write history | Commands panel → list all | Shows history of all writes with status |
| C6 | Write to offline device | Stop simulator → attempt write | Command queued, replayed on reconnect |

### 13.6 Device Management (Per Protocol)

Test each protocol's device manager:

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| P1 | **Modbus**: Add device | Settings → Modbus → Add → `127.0.0.1:5020` | Device appears, status turns green |
| P2 | **Modbus**: Scan registers | Device → Scan | Register list populated from simulator |
| P3 | **Modbus**: Start/Stop | Click Start/Stop buttons | Device connects/disconnects |
| P4 | **S7**: Add device | Settings → S7 → Add → `127.0.0.1:10200` | Device connects, rack=0, slot=1 |
| P5 | **S7**: Browse DB | Device → Browse → DB1 | Tags listed with values |
| P6 | **OPC-UA**: Add device | Settings → OPC-UA → Add → `opc.tcp://127.0.0.1:4840` | Device connects |
| P7 | **OPC-UA**: Browse nodes | Device → Browse | Node tree shown |
| P8 | **ENIP**: Add device | Settings → EtherNet/IP → Add → `127.0.0.1:44818` | Device connects |
| P9 | **DNP3**: Add outstation | Settings → DNP3 → Add → `127.0.0.1:20000` | Outstation connects |
| P10 | **BACnet**: Add device | Settings → BACnet → Add → `127.0.0.1:47809` | Device registers |
| P11 | **MQTT**: Start broker | Settings → MQTT Broker → Start | Broker starts on port 1883 |
| P12 | **MQTT**: Publish | `mosquitto_pub -t plts/1/voltage -m 225.3` | Value appears in dashboard |

### 13.7 Tag Browser

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| T1 | Open tag browser | Click "Tags" in header | Tree view of all device tags |
| T2 | Expand device | Click device node | Shows all registers/tags for device |
| T3 | Tag search | Type in search box | Tags filtered by name |
| T4 | Tag path | Click tag → "Show path" | Full path shown (device → register → tag) |

### 13.8 Flow Editor

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| F1 | Open flows | Click "Flows" in header | Empty flow canvas |
| F2 | Add trigger node | Drag "Modbus Trigger" from palette | Node appears on canvas |
| F3 | Add action node | Drag "Write Register" from palette | Node appears |
| F4 | Connect nodes | Drag from trigger output to action input | Connection line drawn |
| F5 | Configure node | Double-click trigger → set device + interval | Node config saved |
| F6 | Save flow | Click "Save" | Flow persisted |
| F7 | Start flow | Click "Start" | Flow status = running, green indicator |
| F8 | Stop flow | Click "Stop" | Flow stops, data no longer flows |
| F9 | Test flow | Click "Test" | One-shot execution with result preview |
| F10 | Import Node-RED | Settings → Import → paste Node-RED JSON | Flow converted and loaded |

### 13.9 OEE & Analytics

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| O1 | OEE panel | Click OEE icon | Shows Availability, Performance, Quality, OEE% |
| O2 | Shift management | OEE → Shifts → add shift | Shift created with start/end times |
| O3 | Downtime logging | OEE → Downtime → "Log Downtime" | Entry created with reason code |
| O4 | Close downtime | Click "Close" on open downtime | Duration calculated, entry closed |
| O5 | Histogram | Analytics → Histogram → select tag | Histogram chart shown |
| O6 | Run chart | Analytics → Run Chart → select tag | Run chart with control limits |
| O7 | SPC chart | Analytics → SPC → select tag | X-bar / R chart with sigma limits |
| O8 | Custom metric | Analytics → Custom → "Create Metric" → formula | Metric calculated and charted |
| O9 | Comparison | Analytics → Compare → two tags | Side-by-side comparison chart |

### 13.10 Asset Inventory

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| AS1 | Open assets | Click "Assets" in header | Asset list with grid |
| AS2 | Add asset | Click "+" → fill name, type, IP, vendor | Asset created |
| AS3 | Edit asset | Click asset → Edit → change location | Updated |
| AS4 | Delete asset | Click asset → Delete → confirm | Removed |
| AS5 | SNMP scan | Assets → SNMP Scan → enter subnet | Devices found and listed |
| AS6 | Import scan result | Click "Import" on scanned device | Device becomes managed asset |
| AS7 | IP conflict check | Assets → IP Conflicts | Shows any duplicate IPs |
| AS8 | Maintenance log | Asset → Maintenance → "Add Entry" | Log entry with technician + notes |
| AS9 | Firmware catalog | Asset → Firmware → "Add Entry" | Firmware version tracked |
| AS10 | NIS2 export | Assets → NIS2 Export | CSV download with compliance fields |
| AS11 | CSV import | Assets → Import → upload CSV | Assets bulk-created |

### 13.11 Security & Compliance

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| S1 | Security audit | Settings → Security Audit | Shows recent security events |
| S2 | Failed login event | Attempt wrong password → check audit | Failed login logged with IP |
| S3 | Config change event | Change a setting → check audit | Change logged with user + before/after |
| S4 | Export audit | Security → Export CSV | CSV downloads |
| S5 | Compliance config | Settings → Compliance → enable electronic signatures | Config saved |
| S6 | E-signature | Make a change → prompted for e-signature | Signature captured with reason |
| S7 | Audit trail | Settings → Audit Log | Chronological list of all changes |

### 13.12 System & Settings

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| SY1 | System info | Click gear icon → System | Shows hostname, OS, CPU, memory, disk |
| SY2 | Memory stats | System → Memory | Shows DB size, hot/warm/cold row counts |
| SY3 | Theme toggle | Click theme icon → Light/Dark/System | Theme changes immediately |
| SY4 | Language switch | Click language selector → Indonesian | All UI labels switch to Bahasa Indonesia |
| SY5 | Language → Chinese | Select 中文 | All labels switch to Chinese |
| SY6 | Language → German | Select Deutsch | All labels switch to German |
| SY7 | Keyboard shortcuts | Settings → Shortcuts → customize | Shortcut saved, works in dashboard |
| SY8 | Push notifications | Settings → Push → enable browser notifications | Browser permission prompt |
| SY9 | API keys | Settings → API Keys → "Generate Key" | Key shown once (copy it) |
| SY10 | Branding | Settings → Branding → upload logo | Logo appears in header |
| SY11 | Config snapshots | Settings → Config Snapshots → "Create" | Snapshot created |
| SY12 | Snapshot rollback | Snapshots → select older → "Rollback" | Config reverted |
| SY13 | Compression stats | System → Compression | Shows hot/warm/cold tier sizes |

### 13.13 Shift Handover

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| SH1 | Open shift panel | Click shift handover icon | Form with Morning/Afternoon/Night |
| SH2 | Fill report | Enter "What Happened", "What's Pending", "What to Watch" | Text saved |
| SH3 | Auto report | Enable "Auto Report" | Shift report auto-generated at shift end |
| SH4 | @mention | Type @username in notes | User notified on next login |

### 13.14 Recipes

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| R1 | Create recipe | Recipes → "Add Recipe" → define setpoints | Recipe saved |
| R2 | Load recipe | Click "Load" on a recipe | Setpoints applied to live tags |
| R3 | Recipe status | After loading | "Recipe Loaded" banner appears |

### 13.15 AI Copilot

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| AI1 | Open copilot | Click AI icon | Copilot panel opens |
| AI2 | AI settings | Copilot → Settings → enter API key | Provider configured |
| AI3 | Generate proposal | Copilot → "Suggest Alarm Thresholds" | Proposal appears |
| AI4 | Review proposal | Click proposal → review | Shows before/after |
| AI5 | Apply proposal | Click "Apply" | Changes committed |

### 13.16 Scripts

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| SC1 | Create script | Scripts → "Add Script" → write WASM logic | Script saved |
| SC2 | Test script | Click "Test" | Output preview shown |
| SC3 | Execution history | Scripts → Executions | Shows past runs with status |
| SC4 | Script in flow | Flow → add "Run Script" node | Script executes as part of flow |

### 13.17 Simulation Mode

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| SM1 | Toggle simulation | Click simulation toggle | Simulation banner appears |
| SM2 | Sim profiles | Simulation → Profiles → "Create Profile" | Profile with value patterns saved |
| SM3 | Go live | Click "Go Live" | Simulation stops, real data resumes |

### 13.18 Data Manager

| # | What to Test | Steps | Expected Result |
|---|-------------|-------|-----------------|
| DM1 | Open data manager | Click data manager icon | Shows all data sources |
| DM2 | Deadband config | Select source → set deadband | Config saved, filtered writes |
| DM3 | Calc builder | Data source → "Add Calculated" → formula | Calculated tag appears |
| DM4 | Data import | Data Manager → Import → upload CSV | Historical data imported |

---

## 9. API Smoke Tests

Quick curl-based tests to verify every endpoint category. Run with `http://localhost:3000`.

### Health & System

```bash
# Health check
curl -s http://localhost:3000/api/v1/health | jq .
# Expected: {"status":"ok"}

# Version
curl -s http://localhost:3000/api/v1/version | jq .
# Expected: {"version":"0.34.0",...}

# System info (requires auth)
curl -s -b cookie.txt http://localhost:3000/api/v1/system | jq .
```

### Auth

```bash
# Login (save cookie)
curl -s -c cookie.txt http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}' | jq .

# Check session
curl -s -b cookie.txt http://localhost:3000/api/v1/auth/me | jq .

# CSRF token
curl -s -b cookie.txt http://localhost:3000/api/v1/auth/csrf-token | jq .

# Logout
curl -s -b cookie.txt -X POST http://localhost:3000/api/v1/auth/logout
```

### Data

```bash
# Live data
curl -s -b cookie.txt http://localhost:3000/api/v1/live | jq .

# History (last hour)
curl -s -b cookie.txt "http://localhost:3000/api/v1/history?device_id=plts_1&tag_name=voltage&from=2026-05-04T00:00:00Z&to=2026-05-04T23:59:59Z" | jq .

# CSV export
curl -s -b cookie.txt "http://localhost:3000/api/v1/export/csv?device_id=plts_1&tag_name=voltage&from=2026-05-04T00:00:00Z&to=2026-05-04T23:59:59Z" -o export.csv

# Screens
curl -s -b cookie.txt http://localhost:3000/api/v1/screens | jq .

# Data sources
curl -s -b cookie.txt http://localhost:3000/api/v1/data-sources | jq .

# Tag tree
curl -s -b cookie.txt http://localhost:3000/api/v1/tags/tree | jq .
```

### Alarms

```bash
# List alarms
curl -s -b cookie.txt http://localhost:3000/api/v1/alarms | jq .

# Alarm summary
curl -s -b cookie.txt http://localhost:3000/api/v1/alarms/summary | jq .

# Acknowledge all
curl -s -b cookie.txt -X POST http://localhost:3000/api/v1/alarms | jq .
```

### Commands

```bash
# List commands
curl -s -b cookie.txt http://localhost:3000/api/v1/commands | jq .

# Write to tag (replace TAG_ID with real ID)
curl -s -b cookie.txt http://localhost:3000/api/v1/tags/TAG_ID/write \
  -H "Content-Type: application/json" \
  -d '{"value": 42.5}' | jq .

# Get command status (replace CMD_ID)
curl -s -b cookie.txt http://localhost:3000/api/v1/commands/CMD_ID | jq .
```

### Devices (per protocol)

```bash
# Modbus devices
curl -s -b cookie.txt http://localhost:3000/api/v1/modbus-devices | jq .

# OPC-UA devices
curl -s -b cookie.txt http://localhost:3000/api/v1/opcua/devices | jq .

# S7 devices
curl -s -b cookie.txt http://localhost:3000/api/v1/s7/devices | jq .

# EtherNet/IP devices
curl -s -b cookie.txt http://localhost:3000/api/v1/enip/devices | jq .

# DNP3 devices
curl -s -b cookie.txt http://localhost:3000/api/v1/dnp3/devices | jq .

# BACnet devices
curl -s -b cookie.txt http://localhost:3000/api/v1/bacnet/devices | jq .
```

### OEE & Analytics

```bash
# OEE metrics
curl -s -b cookie.txt http://localhost:3000/api/v1/oee/metrics | jq .

# Downtime log
curl -s -b cookie.txt http://localhost:3000/api/v1/oee/downtime | jq .

# Shifts
curl -s -b cookie.txt http://localhost:3000/api/v1/oee/shifts | jq .

# Analytics aggregate
curl -s -b cookie.txt "http://localhost:3000/api/v1/analytics/aggregate?tag=voltage&period=1h" | jq .
```

### SSE Stream

```bash
# Subscribe to real-time events (alarms, data, commands)
curl -s -b cookie.txt http://localhost:3000/api/v1/stream
# Expected: SSE stream with event: alarm, event: data, event: command_ack
```

### Assets

```bash
# List assets
curl -s -b cookie.txt http://localhost:3000/api/v1/assets | jq .

# NIS2 export
curl -s -b cookie.txt http://localhost:3000/api/v1/assets/nis2-export -o nis2.csv
```

### HA (High Availability)

```bash
# HA status
curl -s -b cookie.txt http://localhost:3000/api/v1/ha/status | jq .
```

---

## 10. Automated Test Suite

### Rust unit & integration tests

```bash
# Run all 627 tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module
cargo test storage::tests
cargo test modbus
cargo test alarm

# Run only integration tests
cargo test --test '*'

# Run with thread control (for parallel-sensitive tests)
cargo test -- --test-threads=1

# Generate test report
cargo test 2>&1 | tee test-report.txt
```

### Frontend tests

```bash
cd client/

# Type check
npx tsc --noEmit

# Lint
npx eslint src/ --ext .ts,.tsx

# Build (catches most issues)
npm run build
```

### CI pipeline (GitHub Actions)

Located at `.github/workflows/ci.yml`, runs on every push:

| Job | What it checks |
|-----|---------------|
| `rust` | `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build --release` |
| `frontend` | `npm ci`, `npm run build` |
| `worker` | `wrangler deploy --dry-run` |
| `landing` | HTML validation |

```bash
# Simulate CI locally
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo build --release
cd client && npm ci && npm run build
```

---

## 11. CLI Command Tests

Test every CLI subcommand:

```bash
# Build release binary
cargo build --release

# Help
./target/release/voltrus --help
# Expected: full usage text

# Start server
./target/release/voltrus test-config.yaml &
sleep 3
curl -s http://localhost:3000/api/v1/health
kill %1

# License status
./target/release/voltrus --license-status test-config.yaml
# Expected: JSON license status (exit 1 = invalid/expired)

# DB stats
./target/release/voltrus db stats test-config.yaml
# Expected: table with hot/warm/cold row counts

# Backup
./target/release/voltrus backup --config test-config.yaml --output /tmp/test-backup.tar.gz
# Expected: "Backup created: /tmp/test-backup.tar.gz"

# Restore
./target/release/voltrus restore /tmp/test-backup.tar.gz --config test-config.yaml --force
# Expected: "Restore complete"

# Config export
./target/release/voltrus config export /tmp/test-config-export --config test-config.yaml
# Expected: "Config exported to /tmp/test-config-export/"

# Config validate
./target/release/voltrus config validate /tmp/test-config-export
# Expected: "OK" messages, exit 0

# Config import (into fresh DB)
rm -f test-voltrus.db
./target/release/voltrus config import /tmp/test-config-export --config test-config.yaml
# Expected: "Config imported"

# Network discovery (requires license + subnet)
./target/release/voltrus discover --range 127.0.0.1/32 --port 5020 --config test-config.yaml
# Expected: JSON array of found devices

# Factory reset
echo "yes" | ./target/release/voltrus factory-reset test-config.yaml
# Expected: "Database deleted"

# Reset admin password
./target/release/voltrus --reset-admin-password test-config.yaml
# Expected: "New password: <16-char-random>"
```

---

## 12. Protocol-Specific Deep Tests

### 12.1 Modbus TCP

```bash
# Start simulator with faults
cargo run --bin modbus-sim -- --port 5020 --delay 0.1 --disconnect 0.05

# Test 1: Connection resilience
# - Start Voltrus with device pointing to 127.0.0.1:5020
# - Watch logs for reconnection attempts
# - Verify data resumes after reconnect
# Expected: auto-reconnect within 5s, no data gaps > 10s

# Test 2: Register read accuracy
# - Read voltage register via API
# - Compare with simulator's current value (logged in simulator terminal)
# Expected: values match within ±0.01

# Test 3: Multi-device
# - Start two modbus-sim instances on ports 5020 and 5021
# - Configure both in Voltrus
# Expected: both devices show independent data
```

### 12.2 S7 Communication

```bash
# Start simulator
cargo run --bin s7-sim -- --port 10200

# Test 1: DB read
# - Add S7 device 127.0.0.1:10200, rack=0, slot=1
# - Browse DB1
# Expected: 28 float tags listed

# Test 2: Write to DB
# - Write 123.456 to DB1.DBD0 (voltage)
# Expected: value reflected in simulator logs

# Test 3: Connection drop
# - Kill s7-sim → wait 10s → restart
# Expected: device shows offline, then reconnects automatically
```

### 12.3 OPC-UA

```bash
# Start simulator
cargo run --bin opcua-sim -- --port 4840

# Test 1: Connect with no security
# - Add device: opc.tcp://127.0.0.1:4840, security=none
# Expected: connects, nodes browsable

# Test 2: Subscribe to nodes
# - Add nodes to subscription
# Expected: values update every 1s

# Test 3: Browse root folder
# Expected: Server node visible with server status
```

### 12.4 EtherNet/IP

```bash
# Start simulator
cargo run --bin enip-sim -- --port 44818

# Test 1: List identity
# Expected: Device type = CompactLogix, vendor = Allen-Bradley

# Test 2: Read tag
# - Read tag "voltage" from Program:MainProgram
# Expected: float value ~220.0

# Test 3: Write tag
# Expected: value accepted and reflected
```

### 12.5 DNP3

```bash
# Start outstation
cargo run --bin dnp3-sim -- 20000 10

# Test 1: Connect master
# - Add DNP3 device 127.0.0.1:20000, address=1, outstation=10
# Expected: connects, points readable

# Test 2: Read analog inputs
# Expected: voltage, current, power values returned

# Test 3: Scan for outstations
# - Use DNP3 scan endpoint
# Expected: outstation discovered at 127.0.0.1:20000
```

### 12.6 BACnet/IP

```bash
# Start simulator
cargo run --bin bacnet-sim -- --port 47809 --device 101

# Test 1: Who-Is/I-Am
# Expected: device responds to BACnet discovery

# Test 2: ReadProperty
# Expected: analog input values returned

# Test 3: ReadPropertyMultiple
# Expected: multiple properties in single request
```

### 12.7 MQTT

```bash
# Start broker simulator
cargo run --bin mqtt-sim -- --port 1883

# Test 1: Connect with MQTT client
mosquitto_sub -h 127.0.0.1 -p 1883 -t "plts/#" -v
# Expected: sees periodic energy data publications

# Test 2: Start Voltrus embedded broker
# - Configure mqtt_broker in config
# Expected: broker starts, devices can publish

# Test 3: Subscribe from Voltrus
# - Create data source with MQTT source_type
# Expected: values from MQTT appear in dashboard
```

---

## 13. Performance & Load Testing

### 13.1 Sustained load (10,000 tags @ 1Hz)

This is the target performance budget: 10K tags, 1 reading/second, on 1 vCPU / 512MB RAM.

```bash
# Start 5 Modbus simulators with many registers
for port in 5020 5021 5022 5023 5024; do
  cargo run --bin modbus-sim -- --port $port &
done

# Create config with 50 devices × 200 registers each = 10,000 tags
# (generate config programmatically or use API to bulk-create data sources)

# Monitor:
# - CPU usage: top -pid $(pgrep voltrus)
# - Memory: watch 'ps aux | grep voltrus'
# - DB size: ls -lh voltrus.db
# - Response time: curl -o /dev/null -s -w '%{time_total}' http://localhost:3000/api/v1/live
```

**Targets:**
- CPU < 80% sustained
- Memory < 512MB
- API response < 500ms for `/api/v1/live`
- No data gaps > 2s in SSE stream

### 13.2 API load test with `wrk` or `hey`

```bash
# Install hey (Go-based HTTP load tester)
# go install github.com/rakyll/hey@latest

# Login and save token
TOKEN=$(curl -s http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-pass"}' | jq -r '.token')

# Load test /api/v1/live (100 concurrent, 30 seconds)
hey -z 30s -c 100 -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/v1/live

# Expected: > 100 req/s, p99 < 500ms, 0 errors

# Load test /api/v1/health (unauthenticated)
hey -z 30s -c 200 http://localhost:3000/api/v1/health
# Expected: > 500 req/s, p99 < 100ms
```

### 13.3 Database growth test

```bash
# Run for 24 hours with 10K tags @ 1Hz = 864M rows
# Expected: ~15GB database size with compression enabled

# Check compression stats
curl -s -b cookie.txt http://localhost:3000/api/v1/system/memory | jq .
```

### 13.4 Frontend performance

```bash
# Build production bundle
cd client && npm run build

# Check bundle sizes
ls -lh dist/assets/*.js
# Targets:
#   index.js < 1.5MB (main bundle)
#   three.js < 800KB (lazy, loaded only for 3D view)
#   ScadaView.js < 400KB (lazy)
#   DigitalTwinView.js < 250KB (lazy)

# Lighthouse test (Chrome DevTools)
# Targets:
#   First Contentful Paint < 2s
#   Time to Interactive < 3s
#   Lighthouse Performance > 80
```

---

## 14. Disaster Recovery Tests

### 14.1 Backup & Restore

```bash
# Step 1: Create some data (run for 1 hour with simulators)
# Step 2: Create backup
voltrus backup --config test-config.yaml --output pre-test-backup.tar.gz

# Step 3: Make changes (add devices, users, alarms)
# Step 4: Restore backup
voltrus restore pre-test-backup.tar.gz --config test-config.yaml --force

# Step 5: Verify restored state
# - Check devices list matches backup time
# - Check user count matches
# - Check data history available
```

### 14.2 Database corruption recovery

```bash
# Step 1: Stop Voltrus
# Step 2: Corrupt DB
echo "corrupt" >> test-voltrus.db

# Step 3: Start Voltrus
# Expected: graceful error message, not a panic

# Step 4: Factory reset
echo "yes" | voltrus factory-reset test-config.yaml

# Step 5: Restore from backup
voltrus restore pre-test-backup.tar.gz --config test-config.yaml --force
```

### 14.3 Simulator crash during operation

```bash
# Step 1: Start Voltrus + Modbus simulator
# Step 2: Kill simulator (kill -9)
# Expected: device goes offline in dashboard, alarm triggers
# Step 3: Restart simulator
# Expected: device reconnects, data resumes, alarm auto-clears
```

### 14.4 Voltrus crash recovery

```bash
# Step 1: Start Voltrus + simulators
# Step 2: Wait for data collection
# Step 3: Kill Voltrus (kill -9)
# Step 4: Restart Voltrus
# Expected: all data preserved, devices reconnect, no data loss
```

### 14.5 Full disk scenario

```bash
# Step 1: Fill disk to 95%
dd if=/dev/zero of=/tmp/bigfile bs=1M count=X  # adjust X to fill disk

# Step 2: Run Voltrus
# Expected: graceful degradation, error in logs, no data corruption

# Step 3: Remove bigfile
rm /tmp/bigfile

# Step 4: Check DB integrity
sqlite3 test-voltrus.db "PRAGMA integrity_check;"
```

---

## 15. Production Readiness Checklist

### Security

- [ ] Change default `auth_secret` in config.yaml to a strong random string
- [ ] Admin password changed from auto-generated default
- [ ] All default passwords rotated
- [ ] HTTPS configured (reverse proxy with Let's Encrypt or self-signed)
- [ ] Firewall rules: only ports 80/443 exposed
- [ ] No unnecessary ports open (simulators should NOT be internet-facing)
- [ ] API keys rotated regularly
- [ ] Role-based access configured (not everyone as admin)
- [ ] Write protection enabled on critical devices
- [ ] Electronic signatures enabled for compliance (if required)
- [ ] Audit log reviewed regularly

### Reliability

- [ ] Backup cron job configured (daily minimum)
  ```bash
  0 2 * * * /usr/local/bin/voltrus backup --config /etc/voltrus/prod.yaml --output /backup/voltrus-$(date +\%Y\%m\%d).tar.gz
  ```
- [ ] Backup restore tested at least once
- [ ] HA configuration tested (failover works)
- [ ] Watchdog process monitoring Voltrus health
- [ ] Disk space monitoring (alert at 80%)
- [ ] Memory monitoring (alert at 80% of available)

### Performance

- [ ] Load tested at expected tag count
- [ ] API response times acceptable (< 500ms for live data)
- [ ] Database growth rate acceptable
- [ ] Compression enabled and working (check hot/warm/cold tiers)
- [ ] Frontend loads in < 3 seconds
- [ ] SSE stream stable (no disconnects under load)

### Monitoring

- [ ] Health endpoint monitored: `GET /api/v1/health`
- [ ] License expiry alerting (30 days before expiry)
- [ ] Alarm notification channels tested (email/push)
- [ ] Database size monitoring
- [ ] Log aggregation configured (journald or file-based)

### Documentation

- [ ] Runbook for common operations (backup, restore, restart, reset password)
- [ ] Network diagram showing Voltrus + PLCs + firewall rules
- [ ] Config file documented and version-controlled
- [ ] Known issues / limitations documented

### Pre-Deployment

- [ ] All manual test matrix items pass (Section 3)
- [ ] All API smoke tests pass (Section 4)
- [ ] All automated tests pass: `cargo test` (627 tests)
- [ ] All clippy warnings resolved: `cargo clippy -- -D warnings`
- [ ] All CLI commands tested (Section 6)
- [ ] Performance targets met (Section 8)
- [ ] Disaster recovery tested (Section 9)
- [ ] License activated and verified
- [ ] Rollback plan documented (how to revert to previous version)

---

## Appendix A: Scenario Launcher

The `run-scenario.sh` script handles all tmux setup. See Section 1 for usage.

```bash
# List scenarios
./run-scenario.sh            # prints help

# Launch a study case (starts all simulators + Voltrus in tmux)
./run-scenario.sh solar-farm
./run-scenario.sh water-treatment
./run-scenario.sh factory-floor
./run-scenario.sh data-center
./run-scenario.sh oil-gas-well

# Stop
tmux kill-session -t voltrus-solar-farm
```

Each scenario directory contains:

```
scenarios/<name>/
├── start.sh         ← starts all simulators in tmux windows
└── voltrus.yaml     ← Voltrus config (ports, devices, registers)
```

The old generic `start-sims.sh` is still available for starting all 7
simulators with default profiles (no scenario-specific data).

## Appendix B: Quick Health Check Script

Save as `health-check.sh`:

```bash
#!/usr/bin/env bash
# Quick health check for all Voltrus components
set -e

HOST="${1:-http://localhost:3000}"

echo "=== Voltrus Health Check ==="
echo ""

# 1. Server health
echo -n "Server:        "
STATUS=$(curl -s -o /dev/null -w '%{http_code}' "$HOST/api/v1/health")
if [ "$STATUS" = "200" ]; then echo "✅ OK ($STATUS)"; else echo "❌ FAIL ($STATUS)"; fi

# 2. Version
echo -n "Version:       "
VERSION=$(curl -s "$HOST/api/v1/version" 2>/dev/null | jq -r '.version' 2>/dev/null || echo "error")
echo "$VERSION"

# 3. Modbus simulators (check if ports are listening)
echo -n "Modbus sim:    "
if nc -z 127.0.0.1 5020 2>/dev/null; then echo "✅ Running (port 5020)"; else echo "⚠️  Not running"; fi

echo -n "S7 sim:        "
if nc -z 127.0.0.1 10200 2>/dev/null; then echo "✅ Running (port 10200)"; else echo "⚠️  Not running"; fi

echo -n "OPC-UA sim:    "
if nc -z 127.0.0.1 4840 2>/dev/null; then echo "✅ Running (port 4840)"; else echo "⚠️  Not running"; fi

echo -n "ENIP sim:      "
if nc -z 127.0.0.1 44818 2>/dev/null; then echo "✅ Running (port 44818)"; else echo "⚠️  Not running"; fi

echo -n "DNP3 sim:      "
if nc -z 127.0.0.1 20000 2>/dev/null; then echo "✅ Running (port 20000)"; else echo "⚠️  Not running"; fi

echo -n "BACnet sim:    "
if nc -z 127.0.0.1 47809 2>/dev/null; then echo "✅ Running (port 47809)"; else echo "⚠️  Not running"; fi

echo -n "MQTT sim:      "
if nc -z 127.0.0.1 1883 2>/dev/null; then echo "✅ Running (port 1883)"; else echo "⚠️  Not running"; fi

# 4. Database size
echo ""
echo -n "Database:      "
if [ -f "test-voltrus.db" ]; then
  SIZE=$(du -h test-voltrus.db | cut -f1)
  echo "✅ $SIZE"
else
  echo "⚠️  test-voltrus.db not found (use voltrus.db if different)"
fi

echo ""
echo "=== Done ==="
```

## Appendix C: Test Automation Script

Save as `run-all-tests.sh`:

```bash
#!/usr/bin/env bash
# Run the full Voltrus test suite
set -e

echo "╔══════════════════════════════════════════════════╗"
echo "║         Voltrus Full Test Suite                   ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""

FAIL=0

echo "1. Formatting check..."
cargo fmt -- --check || { echo "❌ cargo fmt failed"; FAIL=1; }
echo "   ✅ Pass"

echo "2. Clippy (strict)..."
cargo clippy -- -D warnings || { echo "❌ cargo clippy failed"; FAIL=1; }
echo "   ✅ Pass"

echo "3. Rust tests (627 expected)..."
cargo test || { echo "❌ cargo test failed"; FAIL=1; }
echo "   ✅ Pass"

echo "4. Release build..."
cargo build --release || { echo "❌ cargo build failed"; FAIL=1; }
echo "   ✅ Pass"

echo "5. Frontend build..."
cd client && npm ci && npm run build || { echo "❌ npm build failed"; FAIL=1; }
cd ..
echo "   ✅ Pass"

echo ""
if [ $FAIL -eq 0 ]; then
  echo "╔══════════════════════════════════════════════════╗"
  echo "║  ✅ ALL TESTS PASSED — Ready for deployment      ║"
  echo "╚══════════════════════════════════════════════════╝"
else
  echo "╔══════════════════════════════════════════════════╗"
  echo "║  ❌ SOME TESTS FAILED — Do not deploy             ║"
  echo "╚══════════════════════════════════════════════════╝"
  exit 1
fi
```

---

## 16. Real Device References

Device profiles in `src/sim_profiles.rs` are based on actual hardware datasheets.

| Profile | Real Device | Key Specs | Datasheet Reference |
|---------|-------------|-----------|-------------------|
| SMA Inverter | Sunny Tripower STP 60-US-10 | 60kW, 2 MPP trackers, 98.6% peak efficiency, 800V DC max | SMA Solar Technology AG |
| PM5560 | Schneider Electric PM5560 | 0.2S accuracy class, CT/VT connected, revenue-grade energy, 600V L-L | Schneider Electric |
| Promag 10W | Endress+Hauser Promag 10W DN50 | Electromagnetic, 0–50 m³/h, ±0.5% of reading, IP67 | Endress+Hauser |
| AZL402 | ABB AZL402 Navigator | pH/ORP/turbidity/DO/conductivity/free chlorine, multi-parameter | ABB Measurement & Analytics |
| Daniel 1000 | Emerson Daniel 1000 | Orifice flow computer, AGA3/AGA8, gas chromatograph input | Emerson Automation Solutions |
| PX3-5902V | Raritan PX3 5902V | 3-phase 32A, 24 outlets, per-outlet metering, environmental sensors | Raritan (Legrand) |
| CompactLogix | Allen-Bradley 1769-L33ER | 2MB memory, 16 I/O modules, EtherNet/IP, CIP | Rockwell Automation |
| SEL-351S | Schweitzer SEL-351S | Overcurrent, frequency, breaker failure, reclosing, DNP3 | Schweitzer Engineering Laboratories |
| Tracer SC+ | Trane Tracer SC+ | BACnet/IP controller, AHU/VAV/chiller, 50 objects typical | Trane Technologies |
| S7-1200 | Siemens SIMATIC S7-1200 | 14-bit analog inputs, Profinet, up to 6 DBs | Siemens AG |
| Kipp&Zonen SMP10 | Pyranometer | ISO 9060 Class A, 0–4000 W/m², <1% directional error | Kipp&Zonen (OTT HydroMet) |
| Vaisala WXT536 | Weather transmitter | Wind speed/direction, T, RH, rain, pressure, SDI-12 | Vaisala Oyj |
| E+H Liquiphant | Level switch | Vibrating fork, 4–20mA, SIL 2/3 | Endress+Hauser |
| ROC800 | Emerson ROC800 | Remote operations controller, DNP3, Modbus, flow arithmetic | Emerson Automation Solutions |

### Physical constants used

| Parameter | Value | Standard |
|-----------|-------|----------|
| Nominal L-N voltage | 230V | IEC 60038 (230V ±10%) |
| Nominal L-L voltage | 400V | 230V × √3 |
| Grid frequency | 50Hz | IEC 60038 |
| Power factor range | 0.80–0.99 lead/lag | Typical PFC-corrected industrial |
| Voltage THD limit | <5% | IEC 61000-2-4 Class 2 |
| Current THD range | 5–15% | Typical VFD-heavy load |
| Water pH range | 6.5–8.5 | WHO Guidelines for Drinking-Water Quality |
| Turbidity treated | <1 NTU | WHO Guideline |
| Chlorine residual | 0.2–0.5 mg/L | WHO Guideline |
| CT ratio (PM5560) | 2000:5A | Typical LV feeder |
