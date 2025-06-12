use std::error::Error;
use serde::Serialize;
use csv::Writer;
#[cfg(target_os = "linux")]
use perf_event::Builder;
use risc0_zkvm::Receipt;

use postcard;

#[derive(Serialize, Debug)] 
pub struct HostMetrics {
    run_id: String, 
    csv_file_path: String, 
    proving_time_seconds: f64, 
    inputs_size_bytes: u64,
    proof_size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu_cycles_host: Option<u64>,
    guest_cycles: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    segments: Option<u64>, 
    #[serde(skip_serializing_if = "Option::is_none")]
    total_cycles: Option<u64>, 
}

impl HostMetrics {
    pub fn new(csv_file_path: String, run_id: String) -> Self { 
        Self {
            run_id,
            csv_file_path,
            proving_time_seconds: 0.0,
            inputs_size_bytes: 0,
            proof_size_bytes: 0,
            cpu_cycles_host: None,
            guest_cycles: 0,
            segments: None,
            total_cycles: None,
        }
    }

    pub fn runtime(&mut self, duration_secs: f64) {
        self.proving_time_seconds = duration_secs;
    }

    pub fn proof_size(&mut self, receipt: &Receipt) {
        match postcard::to_allocvec(receipt) { 
            Ok(bytes) => self.proof_size_bytes = bytes.len() as u64,
            Err(_) => {
                eprintln!("Warning: Failed to serialize receipt for proof_size calculation.");
                self.proof_size_bytes = 0; 
            }
        }
    }

    pub fn input_size(&mut self, size_bytes: u64) {
        self.inputs_size_bytes = size_bytes;
    }

    #[cfg(target_os = "linux")]
    pub fn host_cpu_cycles<F>(&mut self, f: F) -> Result<(), Box<dyn Error>>
    where
        F: FnOnce(),
    {
        let mut counter = Builder::new().build()?;
        counter.enable()?;
        f();
        counter.disable()?;
        self.cpu_cycles_host = Some(counter.read()?);
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn host_cpu_cycles<F>(&mut self, f: F) -> Result<(), Box<dyn Error>>
    where
        F: FnOnce(),
    {
        f(); 
        self.cpu_cycles_host = None;
        Ok(())
    }
    
    pub fn segments(&mut self, receipt: &Receipt) { 
        self.segments = None;
        eprintln!("Warnung: Segment-Extraktion in host_metrics.rs muss für risc0-zkvm v2.1.x API überprüft werden.");
    }

    pub fn total_cycles(&mut self, receipt: &Receipt) {
        self.total_cycles = None;
        eprintln!("Warnung: Zyklus-Extraktion in host_metrics.rs muss für risc0-zkvm v2.1.x API überprüft werden.");
    }

    pub fn metrics_write_csv(&self) -> Result<(), Box<dyn Error>> {
        let file_exists = std::path::Path::new(&self.csv_file_path).exists();
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&self.csv_file_path)?; 

        let mut wtr = Writer::from_writer(file);

        if !file_exists {
            wtr.write_record(&[
                "run_id", 
                "proving_time_seconds",
                "inputs_size_bytes",
                "proof_size_bytes",
                "cpu_cycles_host",
                "guest_cycles",
                "segments",
                "total_cycles",
            ])?;
        }
        
        wtr.serialize(self)?;
        wtr.flush()?;
        Ok(())
    }

    pub fn guest_cycles(&mut self, risc_v_cycles: u64) {
        self.guest_cycles = risc_v_cycles;
    }
}