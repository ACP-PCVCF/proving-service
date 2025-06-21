extern crate alloc;

use alloc::{vec::Vec, string::String, format};
use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey};
use rsa::pkcs1v15::Pkcs1v15Sign;
use sha2::{Sha256, Digest as Sha2DigestTrait};
use base64::{engine::general_purpose, Engine as _};
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

risc0_zkvm::guest::entry!(main);
 
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct GuestMetrics {
    pub start_cycles: u64,
    pub end_cycles: u64,
    pub risc_v_cycles: u64,
}

impl GuestMetrics {
    pub fn new() -> Self {
        Self {
            start_cycles: 0,
            end_cycles: 0,
            risc_v_cycles: 0,
        }
    }

    pub fn start_riscv_cyc_count(&mut self) {
        self.start_cycles = env::cycle_count(); 
    }

    pub fn end_riscv_cyc_count(&mut self) {
        self.end_cycles = env::cycle_count(); 
        self.risc_v_cycles = self.end_cycles.saturating_sub(self.start_cycles);
    }
}

#[derive(Deserialize, Serialize)]
struct Distance {
    actual: f64,
    gcd: Option<f64>, 
    sfd: Option<f64>, 
}

#[derive(Deserialize, Serialize)]
struct SensorDataPayload {
    distance: Distance,
}

#[derive(Deserialize, Serialize)]
//struct Shipment {
struct SignedSensorData {
    //shipment_id: String,
    //info: ShipmentInfo,
    //#[serde(rename = "tceId")]
    tce_id: String,
    //#[serde(rename = "camundaProcessInstanceKey")]
    camunda_process_instance_key: String,
    //#[serde(rename = "camundaActivityId")]
    camunda_activity_id: String,
    sensorkey: String,
    //#[serde(rename = "signedSensorData")]
    signed_sensor_data: String,
    //#[serde(rename = "sensorData")]
    //sensor_data: SensorDataPayload,
    sensor_data: String,
    sensor_data_hash: String,
}

#[derive(Deserialize, Serialize)]
struct Activity {
    process_id: String,
    unit: String,
    consumption: u32,
    e_type: String,
}


#[derive(Deserialize, Serialize)]
struct CombinedInput {
    activities: Vec<Activity>,
    signatures: Vec<SignedSensorData>,
}


fn verify_hash(hash: &str, data: &str) -> bool {
    let mut hasher = Sha256::new();
    Update::update(&mut hasher, data.as_bytes());
    let computed_hash = hasher.finalize();
    let computed_hash_b64 = general_purpose::STANDARD.encode(computed_hash);
    if computed_hash_b64 == hash {
        true
    } else {
        env::log(&format!("Hash-Vergleich fehlgeschlagen. Erwartet: '{}', Berechnet: '{}'", hash, computed_hash_b64));
        false
    }
}

fn main() {
    // Metriken initialisieren
    let mut guest_metrics = GuestMetrics::new();

    let input: CombinedInput = env::read();
    let valid_activities: Vec<Activity> = input.activities; 

    // Start der Zykluszählung
    guest_metrics.start_riscv_cyc_count();
        

    for signature in input.signatures {
        if verify_hash(&signature.sensor_data_hash, &signature.sensor_data)  {
            //env::log(format!("Shipment {}: GÜLTIG", shipment.shipment_id).as_str());
            env::log(format!("Erfolg, Hashes sind Identisch").as_str());
        } else {
            //env::log(format!("Shipment {}: UNGÜLTIG", shipment.shipment_id).as_str());
            env::log(format!("Hash: UNGÜLTIG").as_str());
        }
    }

    let emission_gasoline: u32 = valid_activities
        .iter()
        .filter(|e| e.e_type == "gasoline")
        .map(|e| e.consumption * 2)
        .sum();

    let emission_greenpower: u32 = valid_activities
        .iter()
        .filter(|e| e.e_type == "green power")
        .map(|e| e.consumption * 304)
        .sum();

    let emission_diesel: u32 = valid_activities
        .iter()
        .filter(|e| e.e_type == "diesel")
        .map(|e| e.consumption * 274)
        .sum();

    let pcf_total: u32 = emission_diesel + emission_gasoline + emission_greenpower;

    env::log(format!("PCF total (kg CO2e): {}", pcf_total).as_str());

    guest_metrics.end_riscv_cyc_count();

    env::commit(&(&pcf_total, guest_metrics));
}

#[cfg(test)]
mod tests {
    use super::*;
    use risc0_zkvm::guest::env;

    #[test]
    fn test_guest_metrics_cycle_count() {
        let mut gm = GuestMetrics::new();

        // Simulate start
        gm.start_riscv_cyc_count();
        // Simulated work: no-op
        gm.end_riscv_cyc_count();

        // Check that end > start and cycles computed
        assert!(gm.end_cycles >= gm.start_cycles);
        assert_eq!(gm.risc_v_cycles, gm.end_cycles - gm.start_cycles);
    }
}