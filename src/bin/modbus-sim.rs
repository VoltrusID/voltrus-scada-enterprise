use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use rand::Rng;
use tokio::net::{TcpListener, TcpStream};
use tokio_modbus::prelude::*;
use tokio_modbus::server::Service;
use tokio_modbus::server::tcp::{Server, accept_tcp_connection};

// --- Config ---

struct FaultConfig {
    delay: f64,
    disconnect: f64,
    garbage: f64,
}

// --- Helpers ---

fn f32_to_regs_be(val: f32) -> [u16; 2] {
    let bits = val.to_bits();
    [(bits >> 16) as u16, bits as u16]
}

// --- Data Store ---
//
// Register map (typical 3-phase energy meter, Float32 Big-Endian):
//
//   Address  Size  Name               Description
//   ───────  ────  ─────────────────  ──────────────────────────────
//    0- 1      2   voltage_l1         L1-N voltage (V)
//    2- 3      2   voltage_l2         L2-N voltage (V)
//    4- 5      2   voltage_l3         L3-N voltage (V)
//    6- 7      2   current_l1         L1 current (A)
//    8- 9      2   current_l2         L2 current (A)
//   10-11      2   current_l3         L3 current (A)
//   12-13      2   power_l1           L1 active power (W)
//   14-15      2   power_l2           L2 active power (W)
//   16-17      2   power_l3           L3 active power (W)
//   18-19      2   power_factor       Overall power factor
//   20-21      2   frequency          Grid frequency (Hz)
//   22-23      2   energy_active      Active energy import (kWh)
//   24-25      2   thd_voltage        Voltage THD average (%)
//   26-27      2   thd_current        Current THD average (%)
//
// Derived values (use expression in data source config):
//   voltage_l12 ≈ {voltage_l1} * 1.732
//   voltage_l23 ≈ {voltage_l2} * 1.732
//   voltage_l31 ≈ {voltage_l3} * 1.732
//   power_total = {power_l1} + {power_l2} + {power_l3}
//   current_neutral = sqrt of vector sum (not expressible, use SCADA)
//   reactive/apparent power = computed from power + pf (not expressible)

const NUM_REGS: usize = 28;

struct MeterData {
    voltage: [f32; 3],      // L1-N, L2-N, L3-N
    current: [f32; 3],      // L1, L2, L3
    power_factor: [f32; 3], // per-phase
    frequency: f32,
    energy_active: f32, // kWh (monotonically increasing)
    thd_voltage: [f32; 3],
    thd_current: [f32; 3],
    load_imbalance: f32, // 0.0–1.0 fraction of max imbalance
}

impl MeterData {
    fn new() -> Self {
        Self {
            voltage: [220.0, 219.0, 221.0],
            current: [45.0, 42.0, 47.0],
            power_factor: [0.92, 0.91, 0.93],
            frequency: 50.0,
            energy_active: 158732.0,
            thd_voltage: [1.8, 2.1, 1.6],
            thd_current: [4.5, 5.2, 3.9],
            load_imbalance: 0.1,
        }
    }

    fn to_registers(&self) -> Vec<u16> {
        let mut regs = vec![0u16; NUM_REGS];
        let mut i = 0;

        // Phase voltages L-N (0..5)
        for v in &self.voltage {
            let r = f32_to_regs_be(*v);
            regs[i] = r[0];
            regs[i + 1] = r[1];
            i += 2;
        }
        // Phase currents (6..11)
        for c in &self.current {
            let r = f32_to_regs_be(*c);
            regs[i] = r[0];
            regs[i + 1] = r[1];
            i += 2;
        }
        // Per-phase active power (12..17)
        for p in 0..3 {
            let pw = self.voltage[p] * self.current[p] * self.power_factor[p];
            let r = f32_to_regs_be(pw);
            regs[i] = r[0];
            regs[i + 1] = r[1];
            i += 2;
        }
        // Overall power factor (18..19)
        {
            let total_p: f32 = (0..3)
                .map(|p| self.voltage[p] * self.current[p] * self.power_factor[p])
                .sum();
            let total_s: f32 = (0..3).map(|p| self.voltage[p] * self.current[p]).sum();
            let avg_pf = if total_s > 0.0 {
                total_p / total_s
            } else {
                0.0
            };
            let r = f32_to_regs_be(avg_pf);
            regs[i] = r[0];
            regs[i + 1] = r[1];
            i += 2;
        }
        // Frequency (20..21)
        {
            let r = f32_to_regs_be(self.frequency);
            regs[i] = r[0];
            regs[i + 1] = r[1];
            i += 2;
        }
        // Energy active kWh (22..23)
        {
            let r = f32_to_regs_be(self.energy_active);
            regs[i] = r[0];
            regs[i + 1] = r[1];
            i += 2;
        }
        // THD voltage average (24..25)
        {
            let avg: f32 = self.thd_voltage.iter().sum::<f32>() / 3.0;
            let r = f32_to_regs_be(avg);
            regs[i] = r[0];
            regs[i + 1] = r[1];
            i += 2;
        }
        // THD current average (26..27)
        {
            let avg: f32 = self.thd_current.iter().sum::<f32>() / 3.0;
            let r = f32_to_regs_be(avg);
            regs[i] = r[0];
            regs[i + 1] = r[1];
            i += 2;
        }

        let _ = i;
        regs
    }

    fn tick(&mut self) {
        let mut rng = rand::rng();

        // Voltage: slight per-phase drift + common mode variation
        let v_common = rng.random_range(-1.5..1.5);
        for i in 0..3 {
            let drift = rng.random_range(-0.8..0.8);
            self.voltage[i] = (self.voltage[i] + v_common + drift).clamp(210.0, 230.0);
        }

        // Current: drift with load imbalance
        let i_common = rng.random_range(-2.0..2.0);
        for i in 0..3 {
            let imbalance = self.load_imbalance * rng.random_range(-5.0..5.0);
            self.current[i] = (self.current[i] + i_common + imbalance).clamp(5.0, 80.0);
        }

        // Power factor: slow drift per phase
        for i in 0..3 {
            self.power_factor[i] =
                (self.power_factor[i] + rng.random_range(-0.005..0.005)).clamp(0.80, 0.99);
        }

        // Frequency: tight around 50Hz
        self.frequency = (self.frequency + rng.random_range(-0.005..0.005)).clamp(49.9, 50.1);

        // THD: slight variations
        for i in 0..3 {
            self.thd_voltage[i] =
                (self.thd_voltage[i] + rng.random_range(-0.2..0.2)).clamp(0.5, 5.0);
            self.thd_current[i] =
                (self.thd_current[i] + rng.random_range(-0.3..0.3)).clamp(1.0, 8.0);
        }

        // Energy accumulation
        let total_power: f32 = (0..3)
            .map(|p| self.voltage[p] * self.current[p] * self.power_factor[p])
            .sum();
        self.energy_active += total_power / 3600.0 / 1000.0; // W → kWh (per second tick)
    }
}

// --- Service ---

struct MeterService {
    data: Arc<Mutex<MeterData>>,
    faults: Arc<FaultConfig>,
}

impl Service for MeterService {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ExceptionCode;
    type Future = std::future::Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let should_delay = {
            let mut rng = rand::rng();
            rng.random::<f64>() < self.faults.delay
        };

        if should_delay {
            tracing::warn!("Fault injection: simulated delay on request {req:?}");
        }

        let should_disconnect = {
            let mut rng = rand::rng();
            rng.random::<f64>() < self.faults.disconnect
        };

        if should_disconnect {
            tracing::warn!("Fault injection: simulated disconnect on request {req:?}");
            return std::future::ready(Err(ExceptionCode::ServerDeviceFailure));
        }

        let should_garbage = {
            let mut rng = rand::rng();
            rng.random::<f64>() < self.faults.garbage
        };

        let response = match &req {
            Request::ReadHoldingRegisters(addr, qty) => {
                let data = self.data.lock().unwrap();
                let all_regs = data.to_registers();
                let addr = *addr as usize;
                let qty = *qty as usize;

                let regs: Vec<u16> = if should_garbage {
                    tracing::warn!(
                        "Fault injection: returning garbage data for ReadHoldingRegisters addr={addr} qty={qty}"
                    );
                    let mut rng = rand::rng();
                    (0..qty).map(|_| rng.random::<u16>()).collect()
                } else {
                    let end = std::cmp::min(addr.saturating_add(qty), all_regs.len());
                    all_regs.get(addr..end).unwrap_or(&[]).to_vec()
                };

                Response::ReadHoldingRegisters(regs)
            }
            _ => {
                tracing::warn!("Unsupported request function code: {req:?}");
                return std::future::ready(Err(ExceptionCode::IllegalFunction));
            }
        };

        std::future::ready(Ok(response))
    }
}

// --- CLI ---

fn parse_args() -> (u16, FaultConfig) {
    let args: Vec<String> = std::env::args().collect();
    let mut port: u16 = 5020;
    let mut faults = FaultConfig {
        delay: 0.0,
        disconnect: 0.0,
        garbage: 0.0,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                port = args[i + 1].parse().unwrap_or_else(|_| {
                    eprintln!("Invalid port, using default 5020");
                    5020
                });
                i += 2;
            }
            "--delay" => {
                faults.delay = args[i + 1].parse().unwrap_or(0.0);
                i += 2;
            }
            "--disconnect" => {
                faults.disconnect = args[i + 1].parse().unwrap_or(0.0);
                i += 2;
            }
            "--garbage" => {
                faults.garbage = args[i + 1].parse().unwrap_or(0.0);
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }
    (port, faults)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let (port, faults) = parse_args();
    let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;

    let data = Arc::new(Mutex::new(MeterData::new()));
    let faults = Arc::new(faults);

    // Background task: update values every second
    {
        let data = data.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                let mut d = data.lock().unwrap();
                d.tick();
                tracing::debug!(
                    "V=[{:.1},{:.1},{:.1}] I=[{:.1},{:.1},{:.1}] PF={:.3} F={:.2}Hz E={:.1}kWh",
                    d.voltage[0],
                    d.voltage[1],
                    d.voltage[2],
                    d.current[0],
                    d.current[1],
                    d.current[2],
                    d.power_factor.iter().sum::<f32>() / 3.0,
                    d.frequency,
                    d.energy_active,
                );
            }
        });
    }

    tracing::info!("Modbus 3-phase energy meter simulator listening on {addr}");
    tracing::info!(
        "Fault injection: delay={:.0}% disconnect={:.0}% garbage={:.0}%",
        faults.delay * 100.0,
        faults.disconnect * 100.0,
        faults.garbage * 100.0
    );

    let listener = TcpListener::bind(addr).await?;
    let server = Server::new(listener);

    let service_data = data.clone();
    let service_faults = faults.clone();

    let new_service = move |_socket_addr: SocketAddr| -> io::Result<Option<MeterService>> {
        let service = MeterService {
            data: service_data.clone(),
            faults: service_faults.clone(),
        };
        Ok(Some(service))
    };

    let on_connected = |stream: TcpStream, socket_addr: SocketAddr| {
        let new_service = &new_service;
        async move { accept_tcp_connection(stream, socket_addr, new_service) }
    };

    let on_process_error = |err: io::Error| {
        tracing::error!("Connection error: {err}");
    };

    server.serve(&on_connected, on_process_error).await?;

    Ok(())
}
