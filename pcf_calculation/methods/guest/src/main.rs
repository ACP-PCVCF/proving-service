#![no_main]
#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use base64::{engine::general_purpose, Engine as _};
use bincode;
use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
use sha2::digest::Update;
use sha2::{Digest as Sha2DigestTrait, Sha256};

risc0_zkvm::guest::entry!(main);

#[derive(Deserialize, Serialize)]
struct Activity {
    process_id: String,
    unit: String,
    consumption: u32,
    e_type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct SignedSensorData {
    tce_id: String,
    camunda_process_instance_key: String,
    camunda_activity_id: String,
    sensorkey: String,
    signed_sensor_data: String,
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
struct SignatureForHost {
    commitment: String,
    signature: String,
    pub_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct GuestMetrics {
    pub start_cycles: u64,
    pub end_cycles: u64,
    pub risc_v_cycles: u64,
}

fn hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    Update::update(&mut hasher, data.as_bytes());
    let computed_hash = hasher.finalize();
    general_purpose::STANDARD.encode(computed_hash)
}

fn main() {
    let start_cycles = env::cycle_count();
    let input: CombinedInput = env::read();
    let activities = input.activities;
    let signatures = input.signatures;
    let mut pcf_total: f64 = 0.0;
    for activity in activities {
        pcf_total += activity.consumption as f64;
    }

    let mut signatures_for_host: Vec<SignatureForHost> = Vec::new();
    for signature in signatures {
        let concatenated_data = format!("{}{}", signature.sensor_data, signature.salt);
        let calculated_hash = hash(&concatenated_data);
        assert!(
            calculated_hash == signature.commitment,
            "FEHLER: Commitment stimmt nicht mit dem Hash der Sensordaten überein!"
        );

        signatures_for_host.push(SignatureForHost {
            commitment: signature.commitment,
            signature: signature.signed_sensor_data,
            pub_key: signature.sensorkey,
        });
    }

    let serialized_signatures: Vec<u8> =
        bincode::serialize(&signatures_for_host).expect("Fehler bei der Serialisierung der Signaturen");

    let end_cycles = env::cycle_count();
    let guest_metrics = GuestMetrics {
        start_cycles,
        end_cycles,
        risc_v_cycles: end_cycles.saturating_sub(start_cycles),
    };

    env::commit(&(pcf_total, serialized_signatures, guest_metrics));
}