use std::time::Instant;
use std::error::Error;

use serde::Serialize;
use bincode;
use csv::Writer;
use perf_event::Builder;

use risc0_zkvm::Receipt;

use super::guest::GuestMetrics;

#[derive(Serialize)]
pub struct HostMetrics {
    proving_time: u64, // in milliseconds
    inputs_size: u64,
    proof_size: u64,
    cpu_cyclus_host: u64,
    guest_cycles: u64,
    prove_depth: u64,
    overhead_1: f64,
    efficiency: f64,
}

impl HostMetrics {
    pub fn new() -> Self {
        Self {
            proving_time: 0,
            inputs_size: 0,
            proof_size: 0,
            cpu_cyclus_host: 0,
            guest_cycles: 0,
            prove_depth: 0,
            overhead_1: 0.0,
            efficiency: 0.0,
        }
    }

    pub fn runtime<F, R>(&mut self, _label: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        self.proving_time = (elapsed.as_secs_f64() * 1000.0) as u64;
        result
    }

    pub fn proof_size(&mut self, receipt: &Receipt) {
        self.proof_size = bincode::serialized_size(receipt).unwrap_or(0) as u64;
    }

    pub fn input_size<T: Serialize>(&mut self, input: &T) {
        self.inputs_size = bincode::serialized_size(input).unwrap_or(0) as u64;
    }

    pub fn host_cpu_cycles<F>(&mut self, f: F) -> Result<(), Box<dyn Error>>
    where
        F: FnOnce(),
    {
        let mut counter = Builder::new().build()?;
        counter.enable()?;
        f();
        counter.disable()?;
        self.cpu_cyclus_host = counter.read()?;
        Ok(())
    }

    pub fn metrics_write_csv(&self) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_writer(vec![]);
        wtr.serialize(self)?;
        let data = String::from_utf8(wtr.into_inner()?)?;
        println!("{}", data);
        Ok(())
    }

    pub fn efficiency(&mut self, g_metrics: &GuestMetrics) {
        self.efficiency = self.cpu_cyclus_host as f64 / g_metrics.risc_v_cycles as f64;
    }

    pub fn overhead_1(&mut self) {
        if self.prove_depth > 0 {
            self.overhead_1 = self.proving_time as f64 / self.prove_depth as f64;
        }
    }

    pub fn guest_cycles(&mut self, g_metrics: &GuestMetrics) {
        self.guest_cycles = g_metrics.risc_v_cycles;
    }
}
