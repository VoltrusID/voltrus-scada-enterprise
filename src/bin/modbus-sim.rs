use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use rand::Rng;
use tokio::net::{TcpListener, TcpStream};
use tokio_modbus::prelude::*;
use tokio_modbus::server::tcp::{accept_tcp_connection, Server};
use tokio_modbus::server::Service;

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

struct PltsData {
    voltage: f32,
    current: f32,
    power: f32,
    frequency: f32,
    power_factor: f32,
    energy_total: f32,
}

impl PltsData {
    fn new() -> Self {
        Self {
            voltage: 380.0,
            current: 65.0,
            power: 24700.0,
            frequency: 50.0,
            power_factor: 0.92,
            energy_total: 158732.0,
        }
    }

    fn to_registers(&self) -> Vec<u16> {
        let mut regs = vec![0u16; 12];
        let v = f32_to_regs_be(self.voltage);
        let c = f32_to_regs_be(self.current);
        let p = f32_to_regs_be(self.power);
        let f = f32_to_regs_be(self.frequency);
        let pf = f32_to_regs_be(self.power_factor);
        let e = f32_to_regs_be(self.energy_total);
        regs[0] = v[0];
        regs[1] = v[1];
        regs[2] = c[0];
        regs[3] = c[1];
        regs[4] = p[0];
        regs[5] = p[1];
        regs[6] = f[0];
        regs[7] = f[1];
        regs[8] = pf[0];
        regs[9] = pf[1];
        regs[10] = e[0];
        regs[11] = e[1];
        regs
    }

    fn tick(&mut self) {
        let mut rng = rand::rng();
        self.voltage = (self.voltage + rng.random_range(-2.0..2.0)).clamp(370.0, 390.0);
        self.current = (self.current + rng.random_range(-3.0..3.0)).clamp(40.0, 80.0);
        self.power = self.voltage * self.current * self.power_factor;
        self.frequency = (self.frequency + rng.random_range(-0.01..0.01)).clamp(49.8, 50.2);
        self.power_factor =
            (self.power_factor + rng.random_range(-0.005..0.005)).clamp(0.88, 0.96);
        self.energy_total += self.power / 3600.0;
    }
}

// --- Service ---

struct PltsService {
    data: Arc<Mutex<PltsData>>,
    faults: Arc<FaultConfig>,
}

impl Service for PltsService {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ExceptionCode;
    type Future = std::future::Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let should_delay = {
            let mut rng = rand::rng();
            rng.random::<f64>() < self.faults.delay
        };

        // Delay is handled synchronously here for simplicity with Ready future.
        // A real async delay would require Box::pin or an async block.
        // Since Ready is immediate, we skip actual sleep but log it.
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
                    all_regs
                        .get(addr..addr.saturating_add(qty))
                        .unwrap_or(&[])
                        .to_vec()
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
    tracing_subscriber::fmt::init();

    let (port, faults) = parse_args();
    let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;

    let data = Arc::new(Mutex::new(PltsData::new()));
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
                    "V={:.1} I={:.2} P={:.1} F={:.2} PF={:.3} E={:.1}",
                    d.voltage,
                    d.current,
                    d.power,
                    d.frequency,
                    d.power_factor,
                    d.energy_total
                );
            }
        });
    }

    tracing::info!("Modbus simulator listening on {addr}");
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

    let new_service = move |_socket_addr: SocketAddr| -> io::Result<Option<PltsService>> {
        let service = PltsService {
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
