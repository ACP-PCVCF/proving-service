use methods::{GUEST_CODE_FOR_ZK_PROOF_ELF, GUEST_CODE_FOR_ZK_PROOF_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt, InnerReceipt};
use serde::{Deserialize, Serialize};
use serde_json::{to_string_pretty, from_str};
use std::error::Error;
use std::fs;
use std::time::Instant; 
// use uuid::Uuid; 
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey, pkcs1v15::Pkcs1v15Sign};
use sha2::{Sha256, Digest as Sha2DigestTrait};
use base64::{engine::general_purpose, Engine as _};
use hex;
use bytemuck; 
use const_oid::AssociatedOid;
use pkcs1::ObjectIdentifier;
use digest::{
    self,
    Digest as DigestTrait,
    OutputSizeUser,
    Reset,
    FixedOutputReset,
    generic_array::GenericArray,
    FixedOutput,
    Update
};

use csv::Writer;
#[cfg(target_os = "linux")]
use perf_event::Builder;
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
    sensor_data: String,
    salt: String,
    commitment: String,
}

#[derive(Deserialize, Serialize)]
struct CombinedInput {
    activities: Vec<Activity>,
    signatures: Vec<SignedSensorData>,
}


#[derive(Serialize, Deserialize)]
struct ReceiptExportJson {
    image_id: String,
    receipt: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct GuestMetrics {
    pub start_cycles: u64,
    pub end_cycles: u64,
    pub risc_v_cycles: u64,
}

#[derive(Serialize)]
pub struct HostMetrics {
    proving_time: u64,
    inputs_size: u64,
    proof_size: u64,
    #[cfg(target_os = "linux")] // Conditionally compile the field itself
    cpu_cyclus_host: u64,
    #[cfg(not(target_os = "linux"))] 
    cpu_cyclus_host: Option<u64>, 
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
            #[cfg(target_os = "linux")]
            cpu_cyclus_host: 0,
            #[cfg(not(target_os = "linux"))]
            cpu_cyclus_host: None, 
            guest_cycles: 0,
            prove_depth: 0,
            overhead_1: 0.0,
            efficiency: 0.0,
        }
    }

    pub fn set_proving_time_ms(&mut self, duration_ms: u64) {
        self.proving_time = duration_ms;
    }

    pub fn proof_size(&mut self, receipt: &Receipt) {
        self.proof_size = bincode::serialized_size(receipt).unwrap_or(0) as u64;
    }

    pub fn input_size<T: Serialize>(&mut self, input: &T) {
        self.inputs_size = bincode::serialized_size(input).unwrap_or(0) as u64;
    }

    #[cfg(target_os = "linux")]
    pub fn host_cpu_cycles<F>(&mut self, f: F) -> Result<(), Box<dyn Error>>
    where
        F: FnOnce(),
    {
        use perf_event::Builder;
        let mut counter = Builder::new().build_hardware(perf_event::events::Hardware::CPU_CYCLES)?;
        counter.enable()?;
        f();
        counter.disable()?;
        self.cpu_cyclus_host = counter.read()?;
        Ok(())
    }

    // Method to set prove_depth (segment count)
    pub fn set_prove_depth(&mut self, receipt: &Receipt) {
        match &receipt.inner {
            InnerReceipt::Composite(composite_seal) => {
                self.prove_depth = composite_seal.segments.len() as u64;
            }
            InnerReceipt::Groth16(_) => {
                self.prove_depth = 1; 
            }
            // InnerReceipt::Succinct(_) => {
            //     self.prove_depth = 1;
            // }
            _ => {
                eprintln!("WARN: Unhandled InnerReceipt variant for prove_depth extraction. Prove depth set to 1.");
                self.prove_depth = 1 // Or 1, depending on desired default
            }
        }
    }

    pub fn metrics_write_csv(&self) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_writer(vec![]);
        wtr.serialize(self)?;
        let data = String::from_utf8(wtr.into_inner()?)?;
        println!("{}", data);
        Ok(())
    }

    pub fn efficiency(&mut self, g_metrics: &GuestMetrics) {
        #[cfg(target_os = "linux")]
        if g_metrics.risc_v_cycles > 0 {
            self.efficiency = self.cpu_cyclus_host as f64 / g_metrics.risc_v_cycles as f64;
        } else {
            self.efficiency = 0.0;
        }
        #[cfg(not(target_os = "linux"))]
        if g_metrics.risc_v_cycles > 0 && self.cpu_cyclus_host.is_some() {
             self.efficiency = 0.0;
        } else {
            self.efficiency = 0.0;
        }
    }

    pub fn overhead_1(&mut self) {
        if self.prove_depth > 0 {
            self.overhead_1 = self.proving_time as f64 / self.prove_depth as f64;
        } else {
            self.overhead_1 = 0.0;
        }
    }

    pub fn guest_cycles(&mut self, g_metrics: &GuestMetrics) {
        self.guest_cycles = g_metrics.risc_v_cycles;
    }
}


#[derive(Default, Clone)]
struct Sha256WithOid(Sha256);

impl AssociatedOid for Sha256WithOid {
    const OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");
}

impl OutputSizeUser for Sha256WithOid {
    type OutputSize = <Sha256 as OutputSizeUser>::OutputSize;
}

impl Update for Sha256WithOid {
    fn update(&mut self, data: &[u8]) {
        Update::update(&mut self.0, data);
    }
}

impl FixedOutput for Sha256WithOid {
    fn finalize_into(self, out: &mut GenericArray<u8, Self::OutputSize>) {
        FixedOutput::finalize_into(self.0, out);
    }
}

impl Reset for Sha256WithOid {
    fn reset(&mut self) {
        Reset::reset(&mut self.0);
    }
}

impl FixedOutputReset for Sha256WithOid {
     fn finalize_fixed_reset(&mut self) -> GenericArray<u8, Self::OutputSize> {
        FixedOutputReset::finalize_fixed_reset(&mut self.0)
     }
     fn finalize_into_reset(&mut self, out: &mut GenericArray<u8, Self::OutputSize>) {
        FixedOutputReset::finalize_into_reset(&mut self.0, out);
     }
}

impl DigestTrait for Sha256WithOid {
    fn new() -> Self {
        Sha256WithOid(Sha256::new())
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        Update::update(self, data.as_ref());
    }

    fn finalize(self) -> GenericArray<u8, Self::OutputSize> {
        DigestTrait::finalize(self.0)
    }

    fn new_with_prefix(data: impl AsRef<[u8]>) -> Self {
        Sha256WithOid(Sha256::new_with_prefix(data))
    }

    fn chain_update(self, data: impl AsRef<[u8]>) -> Self {
         Sha256WithOid(self.0.chain_update(data))
    }

    fn finalize_into(self, out: &mut GenericArray<u8, Self::OutputSize>) {
        DigestTrait::finalize_into(self.0, out);
    }

    fn finalize_reset(&mut self) -> GenericArray<u8, Self::OutputSize> {
        DigestTrait::finalize_reset(&mut self.0)
    }

    fn finalize_into_reset(&mut self, out: &mut GenericArray<u8, Self::OutputSize>) {
        DigestTrait::finalize_into_reset(&mut self.0, out);
    }

    fn reset(&mut self) {
        Reset::reset(&mut self.0);
    }

    fn output_size() -> usize {
        <Sha256 as DigestTrait>::output_size()
    }

    fn digest(data: impl AsRef<[u8]>) -> GenericArray<u8, Self::OutputSize> {
        <Sha256 as DigestTrait>::digest(data)
    }
}


fn verify_signature(commitment: &str, signed_sensor_data: &str, sensorkey: &str) -> bool {
    let payload = &commitment;
    let signature_b64 = &signed_sensor_data;
    let public_key_pem = &sensorkey;

    println!("Payload: {}", payload);
    println!("Signature: {}", signature_b64);
    println!("Public Key PEM: {}", public_key_pem);

    let public_key = match RsaPublicKey::from_public_key_pem(public_key_pem) {
        Ok(pk) => pk,
        Err(e) => {
            eprintln!("Fehler beim Laden des Public Keys (SPKI erwartet): {:?}", e);
            match RsaPublicKey::from_pkcs1_pem(public_key_pem) {
                Ok(pk_fallback) => {
                    eprintln!("Warnung: Public Key wurde als PKCS#1 geladen, SPKI wird bevorzugt.");
                    pk_fallback
                },
                Err(e_fallback) => {
                    eprintln!("Fehler beim Laden des Public Keys auch als PKCS#1: {:?}", e_fallback);
                    return false;
                }
            }
        }
    };

    let mut hasher = Sha256::new();
    Update::update(&mut hasher, payload.as_bytes());
    let digest_val = hasher.finalize();

    let signature = match general_purpose::STANDARD.decode(signature_b64) {
        Ok(sig) => sig,
        Err(e) => {
            eprintln!("Fehler beim Dekodieren der Signatur: {:?}", e);
            return false;
        }
    };

    let padding = Pkcs1v15Sign::new::<Sha256WithOid>();
    match public_key.verify(padding, &digest_val, &signature) {
        Ok(_) => {
            println!("Signatur ist gültig.");
            true
        }
        Err(e) => {
            eprintln!("Verifikation fehlgeschlagen: {:?}", e);
            false
        }
    }
}


fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let mut host_metrics = HostMetrics::new();
    
    println!("-------- Host: Lade Inputs --------");
    let activity_json = fs::read_to_string("host/src/activity.json").expect("Konnte activity.json nicht lesen");
    let activities: Vec<Activity> = from_str(&activity_json).expect("Konnte activities nicht parsen");

    let signatures_json = fs::read_to_string("host/src/og2.json").expect("Konnte og2.json nicht lesen");
    let top_level_data_vec: Vec<OgJsonTopLevel> = from_str(&signatures_json).expect("Konnte og2.json nicht parsen");

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
        .expect("Konnte CombinedInput nicht in die ExecutorEnv schreiben")
        .build()
        .expect("Konnte ExecutorEnv nicht bauen");

    println!("-------- Host: Starte Proving Prozess --------");
    let prover = default_prover();
    let prove_start_time = Instant::now();
    let prove_info = prover.prove(env, GUEST_CODE_FOR_ZK_PROOF_ELF).unwrap();
    let prove_duration = prove_start_time.elapsed();
    let receipt = prove_info.receipt;
    println!("-------- Host: Proving erfolgreich abgeschlossen --------");

    let (pcf_total, serialized_signatures, guest_metrics_from_journal): (f64, Vec<u8>, GuestMetrics) =
        receipt.journal.decode().expect("Fehler beim Dekodieren des Journals. Falsches Format?");

    receipt.verify(GUEST_CODE_FOR_ZK_PROOF_ID).unwrap();
    println!("✅ Receipt erfolgreich verifiziert!");
    println!("Guest-Zyklen aus dem Journal: {}", guest_metrics_from_journal.risc_v_cycles);

    #[derive(serde::Deserialize)]
    struct SignatureForHost {
        commitment: String,
        signature: String,
        pub_key: String,
    }

    let signature_data_list: Vec<SignatureForHost> = bincode::deserialize(&serialized_signatures)
        .expect("Bincode-Deserialisierung der Signaturen fehlgeschlagen");

    let mut all_signatures_valid = true;
    if signature_data_list.is_empty() {
        println!("Warnung: Keine Signaturen vom Guest erhalten.");
        all_signatures_valid = false;
    } else {
        for (i, signature_data) in signature_data_list.iter().enumerate() {
            println!("\n--- Verifiziere Signatur {}/{} ---", i + 1, signature_data_list.len());
            if !verify_signature(&signature_data.commitment, &signature_data.signature, &signature_data.pub_key) {
                all_signatures_valid = false; 
            }
        }
    }

    if all_signatures_valid {
        println!("\n✅ Alle Signaturen erfolgreich verifiziert!");
    } else {
        eprintln!("\n❌ Mindestens eine Signatur war ungültig.");
    }

    println!("\nGesamter Product Carbon Footprint: {} kg CO2e", pcf_total);

    let receipt_bytes = bincode::serialize(&receipt).expect("Fehler beim Serialisieren des Receipts");
    let encoded_receipt = general_purpose::STANDARD.encode(receipt_bytes);
    let image_id_hex = hex::encode(bytemuck::cast_slice(&GUEST_CODE_FOR_ZK_PROOF_ID));

    let export_data = ReceiptExportJson {
        image_id: image_id_hex,
        receipt: encoded_receipt,
    };

    let receipt_json = to_string_pretty(&export_data).expect("JSON-Serialisierung fehlgeschlagen");
    fs::write("receipt_output.json", receipt_json).expect("Konnte receipt_output.json nicht schreiben");
    println!("Receipt und Image ID gespeichert in: receipt_output.json");

    host_metrics.set_proving_time_ms((prove_duration.as_secs_f64() * 1000.0) as u64);
    host_metrics.input_size(&combined_input);
    host_metrics.proof_size(&receipt);
    host_metrics.set_prove_depth(&receipt);
    host_metrics.guest_cycles(&guest_metrics_from_journal);
    host_metrics.overhead_1();
    host_metrics.efficiency(&guest_metrics_from_journal);

    if let Err(e) = host_metrics.metrics_write_csv() {
        eprintln!("Fehler beim Schreiben der CSV-Datei: {}", e);
    }

    println!("\n-------- Host: Programm erfolgreich beendet --------");
}
