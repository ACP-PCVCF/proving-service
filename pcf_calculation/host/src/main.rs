use methods::{GUEST_CODE_FOR_ZK_PROOF_ELF, GUEST_CODE_FOR_ZK_PROOF_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, Digest, Receipt}; 
use serde::{Deserialize, Serialize};
use serde_json::{to_string_pretty, from_str};
use std::error::Error;
use std::fs;
use std::time::Instant; 
use uuid::Uuid; 

use csv::Writer;
#[cfg(target_os = "linux")]
use perf_event::Builder;
use postcard;
mod verify;

#[derive(Deserialize, Serialize)]
struct Activity {
    process_id: String,
    unit: String,
    consumption: u32,
    e_type: String,
}

#[derive(Deserialize, Debug)]
struct OgJsonTopLevel {
    #[serde(rename = "productFootprint")]
    product_footprint: serde_json::Value,
    #[serde(rename = "tocData")]
    toc_data: Vec<serde_json::Value>,
    #[serde(rename = "hocData")]
    hoc_data: Vec<serde_json::Value>,
    #[serde(rename = "signedSensorData")]
    signed_sensor_data_list: Vec<SignedSensorData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Distance {
    actual: f64,
    gcd: Option<f64>,
    sfd: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct SignedSensorData {
    #[serde(rename = "tceId")]
    tce_id: String,
    #[serde(rename = "camundaProcessInstanceKey")]
    camunda_process_instance_key: String,
    #[serde(rename = "camundaActivityId")]
    camunda_activity_id: String,
    sensorkey: String,
    #[serde(rename = "signedSensorData")]
    signed_sensor_data: String, 
    #[serde(rename = "sensorData")]
    //sensor_data: SensorDataPayload, 
    sensor_data: String
}

#[derive(Deserialize, Serialize)]
struct CombinedInput {
    activities: Vec<Activity>,
    signatures: Vec<SignedSensorData>,
}

#[derive(Serialize)]
struct ReceiptExport {
    image_id: String,
    receipt: risc0_zkvm::Receipt,
}


#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct GuestMetrics {
    pub start_cycles: u64,
    pub end_cycles: u64,
    pub risc_v_cycles: u64,
}

#[derive(Serialize, Debug)]
pub struct HostMetrics {
    run_id: String,
    csv_file_path: String,
    runtime_prove_s: Option<f64>,
    input_size_bytes: Option<u64>,
    proof_size_bytes: Option<u64>,
    guest_cycles: Option<u64>,
    segments: Option<u64>,
    total_cycles: Option<u64>,
    #[cfg(target_os = "linux")]
    cpu_cycles_host: Option<u64>,
}

impl HostMetrics {
    pub fn new(csv_file_path: String, run_id: String) -> Self {
        Self {
            run_id,
            csv_file_path,
            runtime_prove_s: None,
            input_size_bytes: None,
            proof_size_bytes: None,
            guest_cycles: None,
            segments: None,
            total_cycles: None,
            #[cfg(target_os = "linux")]
            cpu_cycles_host: None,
        }
    }

    pub fn runtime(&mut self, proving_time_seconds: f64) {
        self.runtime_prove_s = Some(proving_time_seconds);
    }

    pub fn input_size(&mut self, inputs_size_bytes: u64) {
        self.input_size_bytes = Some(inputs_size_bytes);
    }

    pub fn proof_size(&mut self, proof: &[u8]) {
        self.proof_size_bytes = Some(proof.len() as u64);
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

    #[cfg(target_os = "linux")]
    pub fn host_cpu_cycles<F>(&mut self, f: F) -> Result<(), Box<dyn Error>>
    where
        F: FnOnce(),
    {
        let mut counter =Builder::new().build()?;
        counter.enable()?;
        f();
        counter.disable()?;
        self.cpu_cyclus_host = counter.read()?;
        Ok(())
    }

    pub fn segments(&mut self, _receipt: &Receipt) { 
        self.segments = None;
        // Beispiel, falls InnerReceipt::Composite(composite_seal) existiert und composite_seal.segments eine Vec ist:
        // if let Ok(inner_receipt) = receipt.inner { // Annahme, dass inner ein Result ist
        //     match inner_receipt {
        //         risc0_zkvm::InnerReceipt::Composite(seal) => self.segments = Some(seal.segments.len() as u64),
        //         _ => self.segments = None,
        //     }
        // }
        eprintln!("INFO: Segment-Extraktion in host_metrics.rs muss für risc0-zkvm API überprüft werden. Currently set to None.");
    }

    pub fn total_cycles(&mut self, _receipt: &Receipt) {
        self.total_cycles = None;
        eprintln!("WARN: host_metrics.total_cycles: Failed to get 'cycles' from 'receipt.metadata'. Field not found. Total cycles set to None.");
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

    pub fn guest_cycles(&mut self, g_metrics: &GuestMetrics) {
        self.guest_cycles = Some(g_metrics.risc_v_cycles);
    }
}


fn main() {

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let run_id = Uuid::new_v4().to_string(); // Corrected: Use imported Uuid
    let mut host_metrics = HostMetrics::new(format!("{}_host_metrics.csv", run_id), run_id.clone());
    
    let activity_json =
        fs::read_to_string("host/src/activity.json").expect("File was not readable!!!");

    let activities: Vec<Activity> = from_str(&activity_json).unwrap();

    let signatures_json =
        fs::read_to_string("host/src/og.json").expect("Sensor file not readable");

    let top_level_data_vec: Vec<OgJsonTopLevel> = from_str(&signatures_json).unwrap();

    let signatures: Vec<SignedSensorData> = if let Some(top_level_data) = top_level_data_vec.get(0) {
        top_level_data.signed_sensor_data_list.clone()
    } else {
        panic!("og.json ist leer oder hat nicht das erwartete Format");
    };

    let combined_input = CombinedInput {
        activities,
        signatures,
    };

    let env = ExecutorEnv::builder()
        .write(&combined_input)
        .expect("Failed to write combined input to ExecutorEnv")
        .build()
        .unwrap();

    let prover = default_prover();

    let prove_start_time = Instant::now();

    let prove_info = prover.prove(env, GUEST_CODE_FOR_ZK_PROOF_ELF).unwrap();

    let prove_duration = prove_start_time.elapsed(); 

    let receipt = prove_info.receipt;

    let (pcf_total, guest_metrics_from_journal): (u32, GuestMetrics) = receipt.journal.decode().unwrap();

    println!("Guest Metrics from Journal: {:?}", guest_metrics_from_journal);

    receipt.verify(GUEST_CODE_FOR_ZK_PROOF_ID).unwrap();

    print!(
        "The total CO2-Emission for the process pID-3423452 is {} kg CO2e",
        { pcf_total }
    );

    let image_id_digest = Digest::from(GUEST_CODE_FOR_ZK_PROOF_ID);
    let image_id_hex = image_id_digest.to_string();

    let export = ReceiptExport {
        image_id: image_id_hex,
        receipt: receipt.clone(),
    };

    let receipt_json = to_string_pretty(&export).expect("JSON serialization failed");

    fs::write("receipt_output.json", receipt_json).expect("Couldn't write receipt_output.json");

    println!("Receipt + Image ID gespeichert in: receipt_output.json");

    if let Err(e) = verify::verify_receipt() {
        eprintln!("❌ Fehler bei der Verifikation: {:?}", e);
    }

    host_metrics.runtime(prove_duration.as_secs_f64()); 

    let serialized_input = postcard::to_allocvec(&combined_input).expect("Failed to serialize combined input");
    let input_size_bytes = serialized_input.len() as u64;
    host_metrics.input_size(input_size_bytes);  

    let serialized_receipt = match postcard::to_allocvec(&receipt) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to serialize receipt: {}", e);
            return;
        }
    };
    host_metrics.proof_size(&serialized_receipt);

    host_metrics.guest_cycles(&guest_metrics_from_journal);
    host_metrics.segments(&receipt); 
    host_metrics.total_cycles(&receipt);

    if let Err(e) = host_metrics.metrics_write_csv() {
        eprintln!("Fehler beim Schreiben der CSV-Datei: {}", e);
    }

    println!("Proof erfolgreich generiert und verifiziert!");
}
