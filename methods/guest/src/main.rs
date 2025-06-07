extern crate alloc;

use alloc::{ vec::Vec, string::String, format };
use risc0_zkvm::guest::env;
use serde::{ Deserialize, Serialize };
use rsa::{ RsaPublicKey, pkcs1::DecodeRsaPublicKey };
use rsa::pkcs1v15::Pkcs1v15Sign;
use sha2::{ Sha256, Digest };
use base64::{ engine::general_purpose, Engine as _ };
use std::{ * };
use serde_json;
use serde_json::{ Value };
use proving_service_core::proofing_document::*;
use proving_service_core::hoc_toc_data::*;
use proving_service_core::product_footprint::*;

//risc0_zkvm::guest::entry!(main);
/* 
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
*/

fn sum_emissions(val: f32, emissions: f32) -> f32 {
    return val + emissions;
}

fn sum_mass(mass: f32, total_mass: f32) -> f32 {
    return total_mass + mass;
}

/*
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
*/

fn main() {
    // Lese die komplexen product footprint daten
    let product_footprint: ProofingDocument = env::read();
    let mut transport_pcf: f64 = 0.0;

    let tces: &Vec<TCE> = &product_footprint.productFootprint.extensions[0].data.tces;

    for tce in tces {
        if tce.tocId.is_some() {
            if let Some(distance) = &tce.distance {
                let emission_factor: f64 = emission_factor_toc(
                    &product_footprint.tocData,
                    tce.tocId.clone().unwrap()
                );
                let emissions: f64 = tce.mass * emission_factor * distance.actual; // TODO: Add here a correct emission factor later
                println!("Emissions from TOC {}: {} kg CO2e", tce.tceId, emissions);
                transport_pcf += emissions;
            } else {
                println!("Distance is missing");
            }
        }

        if tce.hocId.is_some() {
            let emission_factor: f64 = emission_factor_hoc(
                &product_footprint.hocData,
                tce.hocId.clone().unwrap()
            );
            let emissions: f64 = tce.mass * emission_factor; //TODO: Add here the correct emission factor later
            println!("Emissions form HOC {}: {} kg CO2e", tce.tceId, emissions);
            transport_pcf += emissions;
        }
    }

    // TODO: Add later on the indeed verification of the signature of the measurements
    /*
    for shipment in shipments {
        if verify_signature(&shipment.info) {
            env::log(format!("Shipment {}: GÜLTIG", shipment.shipment_id).as_str());

            let data: Value = serde_json::from_str(&shipment.info.activity_data_json).unwrap();
            let shipment_emissions: f64 = data["co2eWTW"].as_str().unwrap().parse::<f64>().unwrap();

            transport_pcf = add_emissions(shipment_emissions, transport_pcf);
        } else {
            env::log(format!("Shipment {}: UNGÜLTIG", shipment.shipment_id).as_str());
        }
    }
    */

    fn add_emissions(emissions: f64, pcf_previous: f64) -> f64 {
        let pcf_new: f64 = pcf_previous + emissions;
        return pcf_new;
    }

    fn emission_factor_toc(toc_data: &Vec<TocData>, toc_id: String) -> f64 {
        let right_toc_data: &TocData = toc_data
            .into_iter()
            .find(|t| { t.tocId == toc_id })
            .unwrap();

        let emission_factor_str: String = right_toc_data.co2eIntensityWTW.clone();

        let factor = emission_factor_str.split(" ").next().unwrap().parse::<f64>().unwrap();

        return factor;
    }

    fn emission_factor_hoc(hoc_data: &Vec<HocData>, hoc_id: String) -> f64 {
        let right_hoc_data: &HocData = hoc_data
            .into_iter()
            .find(|t| { t.hocId == hoc_id })
            .unwrap();

        let emission_factor_str: String = right_hoc_data.co2eIntensityWTW.clone();

        let factor = emission_factor_str.split(" ").next().unwrap().parse::<f64>().unwrap();

        return factor;
    }

    env::log(&format!("Total Emissions {} kg CO2e", transport_pcf));
    env::log(&format!("End of guest programm. Proof can take a while..."));
    env::commit(&transport_pcf);
}
