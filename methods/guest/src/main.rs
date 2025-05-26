extern crate alloc;

use alloc::{vec::Vec, string::String, format};
use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey};
use rsa::pkcs1v15::Pkcs1v15Sign;
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose, Engine as _};
use std::{*}; 
use serde_json; 
use serde_json::{Value}; 


//risc0_zkvm::guest::entry!(main);

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



fn sum_emissions(val:f32, emissions: f32) -> f32 {
       return val + emissions; 
}

fn sum_mass(mass:f32, total_mass:f32) -> f32{
    return total_mass + mass; 
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
    let shipments: Vec<Shipment> = env::read();


    let mut transport_pcf : f32 = 1.0; 
    let mut transport_mass : f32 = 1.0; 
   

    


    for shipment in shipments {        if verify_signature(&shipment.info) {
            env::log(format!("Shipment {}: GÜLTIG", shipment.shipment_id).as_str());
        
        let data : Value = serde_json::from_str(&shipment.info.activity_data_json).unwrap();    
        transport_pcf = sum_emissions(data["co2eWTW"].as_str().unwrap().parse().unwrap(), transport_pcf); 
        transport_mass = sum_mass(data["mass"].as_str().unwrap().parse().unwrap(), transport_mass); 

            //match serde_json::from_str::<Activity>(&shipment.info.activity_data_json) {
            //    Ok(activity) => valid_activities.push(activity),
            //    Err(e) => env::log(format!("Fehler beim Parsen von Activity JSON: {:?}", e).as_str()),
            //}
        } else {
            env::log(format!("Shipment {}: UNGÜLTIG", shipment.shipment_id).as_str());

        
        }
    }

    

    //let result : &str = "result"; 

    let total_emissions_per_kg :f32 = transport_pcf/transport_mass; 

   
    env::log(&format!("Total Emissions {} co2e/kg", total_emissions_per_kg));
    env::commit(&total_emissions_per_kg);
}