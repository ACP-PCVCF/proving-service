extern crate alloc;

use alloc::{vec::Vec, string::String, format};
use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey};
use rsa::pkcs1v15::Pkcs1v15Sign;
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose, Engine as _};


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

    let payload = &info.activity_data_json;
    let signature_b64 = &info.activity_signature;
    let public_key_pem = &info.activity_public_key_pem;

    // Öffentlichen Schlüssel aus PEM extrahieren
    let public_key = match RsaPublicKey::from_pkcs1_pem(public_key_pem) {
        Ok(pk) => pk,
        Err(e) => {
            env::log(format!("Fehler beim Laden des Public Keys: {:?}", e).as_str());
            return false;
        }
    };

    // SHA-256 Hash berechnen
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let digest = hasher.finalize();

    // Signatur base64-dekodieren
    let signature = match general_purpose::STANDARD.decode(signature_b64) {
        Ok(sig) => sig,
        Err(e) => {
            env::log(format!("Fehler beim Dekodieren der Signatur: {:?}", e).as_str());
            return false;
        }
    };

    // Signatur verifizieren
    let padding = Pkcs1v15Sign::new_unprefixed();
    match public_key.verify(padding, &digest, &signature) {
        Ok(_) => {
            env::log("Signatur ist gültig.");
            true
        }
        Err(e) => {
            env::log(format!("Verifikation fehlgeschlagen: {:?}", e).as_str());
            false
        }
    }
}

fn main() {
    // Lese die kombinierte Eingabe (activities und shipments)
    let input: CombinedInput = env::read();

    let valid_activities: Vec<Activity> = input.activities;

    for shipment in input.shipments {
        if verify_signature(&shipment.info) {
            env::log(format!("Shipment {}: GÜLTIG", shipment.shipment_id).as_str());

            //match serde_json::from_str::<Activity>(&shipment.info.activity_data_json) {
            //    Ok(activity) => valid_activities.push(activity),
            //    Err(e) => env::log(format!("Fehler beim Parsen von Activity JSON: {:?}", e).as_str()),
            //}
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