# Voltrus Study Cases

> Industrial simulation scenarios for pre-production testing.
> Each scenario runs **multiple real devices** across different protocols simultaneously.

## Quick Start

```bash
# Build everything
cargo build --release

# Launch a scenario (starts all simulators + Voltrus in tmux)
./run-scenario.sh solar-farm

# List available device profiles
cargo run --bin modbus-sim -- --list

# Dashboard
open http://localhost:3000

# Stop everything
tmux kill-session -t voltrus-solar-farm
```

---

## Scenario 1: Solar Farm — 100kW Rooftop PV Plant

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

**What to test:**
- Inverter DC/AC efficiency tracking
- Grid import/export energy balance
- Irradiance vs power output correlation
- Daily yield accumulation
- Alarm: low irradiance, inverter temperature, grid frequency deviation

**Start:** `./run-scenario.sh solar-farm`

---

## Scenario 2: Water Treatment Plant — Municipal 50,000 m³/d

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

**What to test:**
- Process variable trending (pH, turbidity, chlorine)
- S7 multi-DB read (coagulation DB1 → filtration DB2 → disinfection DB3)
- Filter headloss accumulation and backwash trigger
- BACnet HVAC zone temperature control
- MQTT alarm telemetry (turbidity high, chlorine low)
- OEE: plant throughput vs capacity (50,000 m³/d design)

**Start:** `./run-scenario.sh water-treatment`

---

## Scenario 3: Factory Floor — Food & Beverage Filling Line

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

**What to test:**
- ENIP tag browsing (Program:MainProgram)
- Production line speed tracking (bottles/min)
- CIP cycle phases (rinse → caustic → rinse → acid → sanitize)
- Power demand during CIP heating (spike detection)
- DNP3 outstation communication with relay

**Start:** `./run-scenario.sh factory-floor`

---

## Scenario 4: Data Center — Tier-III, 2MW Critical Load

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

**What to test:**
- Per-outlet PDU power monitoring
- UPS load percentage and battery runtime
- BACnet CRAC supply/return temperature differential
- MQTT environmental alerts (hot aisle >35°C, leak detection, door open)
- OPC-UA DCIM aggregation of all data center metrics
- PUE calculation (Total facility power / IT equipment power)

**Start:** `./run-scenario.sh data-center`

---

## Scenario 5: Oil & Gas — Remote Well Pad

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

**What to test:**
- Gas flow computation (AGA3 orifice: ΔP, P, T → flow rate)
- Gas chromatograph composition (CH₄, CO₂, H₂S)
- Compressor vibration monitoring and oil temperature
- DNP3 dual-outstation polling (SEL relay + ROC RTU)
- MQTT satellite link quality and latency monitoring
- Well pad security (fence breach alarm)

**Start:** `./run-scenario.sh oil-gas-well`

---

## Running Individual Simulators

Each simulator supports `--scenario` and `--role` flags:

```bash
# Run just the SMA inverter on port 5020
cargo run --release --bin modbus-sim -- --scenario solar-farm --role inverter --port 5020

# Run the Daniel flow computer with 5% disconnect faults
cargo run --release --bin modbus-sim -- --scenario oil-gas-well --role gas-flow --port 5020 --disconnect 0.05

# List all available device profiles
cargo run --release --bin modbus-sim -- --list

# Run S7 with water treatment profile
cargo run --release --bin s7-sim -- --scenario water-treatment --port 10200

# Run BACnet with data center CRAC profile
cargo run --release --bin bacnet-sim -- --scenario data-center --port 47809 --device 201

# Run MQTT with oil & gas satellite gateway
cargo run --release --bin mqtt-sim -- --scenario oil-gas-well --port 1883
```

## Fault Injection

All simulators support fault injection for resilience testing:

```bash
# 10% request delay, 2% disconnect, 5% garbage data
cargo run --bin modbus-sim -- --scenario solar-farm --role inverter \
    --delay 0.10 --disconnect 0.02 --garbage 0.05 --port 5020
```

| Flag | Effect | Use Case |
|------|--------|----------|
| `--delay 0.1` | 10% of requests get 500ms delay | Test timeout handling |
| `--disconnect 0.02` | 2% chance of connection drop | Test reconnection logic |
| `--garbage 0.05` | 5% chance of corrupt response | Test data validation |

## Directory Structure

```
voltrus/
├── run-scenario.sh              # Main launcher
├── scenarios/
│   ├── solar-farm/
│   │   ├── start.sh             # Simulator startup for this scenario
│   │   └── voltrus.yaml         # Voltrus config (ports, devices, registers)
│   ├── water-treatment/
│   │   ├── start.sh
│   │   └── voltrus.yaml
│   ├── factory-floor/
│   │   ├── start.sh
│   │   └── voltrus.yaml
│   ├── data-center/
│   │   ├── start.sh
│   │   └── voltrus.yaml
│   └── oil-gas-well/
│       ├── start.sh
│       └── voltrus.yaml
├── src/
│   ├── sim_profiles.rs          # Shared device profiles (all scenarios)
│   └── bin/
│       ├── modbus-sim.rs        # --scenario + --role flag
│       ├── s7-sim.rs
│       ├── enip-sim.rs
│       ├── bacnet-sim.rs
│       ├── mqtt-sim.rs
│       ├── dnp3-sim.rs
│       └── opcua-sim.rs
└── TESTING-STRATEGY.md          # Full testing guide
```

## Real Device References

| Profile | Real Device | Key Specs |
|---------|-------------|-----------|
| SMA Inverter | Sunny Tripower 60TL-US | 60kW, 2 MPP trackers, 98.6% efficiency |
| PM5560 | Schneider PM5560 | 0.2S accuracy, CT/VT, revenue-grade |
| Promag 10W | Endress+Hauser Promag 10W | DN50, 0–50 m³/h, ±0.5% accuracy |
| AZL402 | ABB AZL402 | pH/ORP/turbidity/DO/conductivity/Cl₂ |
| Daniel 1000 | Emerson Daniel 1000 | AGA3/AGA8, gas chromatograph |
| PX3-5902V | Raritan PX3 | 3-phase 32A, per-outlet metering |
| CompactLogix L33ER | Allen-Bradley 1769-L33ER | 2MB memory, 16 I/O modules |
| SEL-351S | Schweitzer SEL-351S | Overcurrent, frequency, breaker failure |
| Tracer SC+ | Trane BACnet controller | AHU, VAV, chiller control |
