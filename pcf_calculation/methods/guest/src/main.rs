extern crate alloc;

use alloc::{vec::Vec, string::String, format};
use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use rsa::{pkcs8::DecodePublicKey, signature::{Verifier}, pss::{Signature, VerifyingKey}};
use hex::decode as hex_decode;
use serde_json;

risc0_zkvm::guest::entry!(main);

#[derive(Deserialize, Serialize)]
struct ShipmentInfo {
    activity_data_json: String,
    activity_signature: String,
    activity_public_key_pem: String,
}

#[derive(Deserialize, Serialize)]
struct Shipment {
    shipment_id: String,
    info: ShipmentInfo,
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
    shipments: Vec<Shipment>,
}

fn verify_signature(info: &ShipmentInfo) -> bool {
    let data_bytes = info.activity_data_json.as_bytes();

    let signature_bytes = match hex_decode(&info.activity_signature) {
        Ok(sig) => sig,
        Err(e) => {
            env::log(format!("Fehler beim Dekodieren der Signatur: {:?}", e).as_str());
            return false;
        }
    };

    let pub_key = match rsa::RsaPublicKey::from_public_key_pem(&info.activity_public_key_pem) {
        Ok(pk) => pk,
        Err(e) => {
            env::log(format!("Fehler beim Laden des Public Keys: {:?}", e).as_str());
            return false;
        }
    };

    let verifying_key = VerifyingKey::<Sha256>::new(pub_key);

    match verifying_key.verify(data_bytes, &Signature::try_from(signature_bytes.as_slice()).unwrap()) {
        Ok(_) => true,
        Err(e) => {
            env::log(format!("Verifikation fehlgeschlagen: {:?}", e).as_str());
            false
        }
    }
}

fn main() {
    // Lese die kombinierte Eingabe (activities und shipments)
    let input: CombinedInput = env::read();

    let mut valid_activities: Vec<Activity> = input.activities;

    for shipment in input.shipments {
        if verify_signature(&shipment.info) {
            env::log(format!("Shipment {}: GÜLTIG", shipment.shipment_id).as_str());

            match serde_json::from_str::<Activity>(&shipment.info.activity_data_json) {
                Ok(activity) => valid_activities.push(activity),
                Err(e) => env::log(format!("Fehler beim Parsen von Activity JSON: {:?}", e).as_str()),
            }
        } else {
            env::log(format!("Shipment {}: UNGÜLTIG", shipment.shipment_id).as_str());
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
    env::commit(&pcf_total);
}