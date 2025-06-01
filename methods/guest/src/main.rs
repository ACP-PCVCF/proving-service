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



#[derive(Debug, Serialize, Deserialize)]
pub struct ProductFootprintRoot {
    pub productFootprint: ProductFootprint,
    pub tocData: Vec<TocData>,
    pub hocData: Vec<HocData>,
    pub signedSensorData: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductFootprint {
    pub id: String,
    pub specVersion: String,
    pub version: i32,
    pub created: String,
    pub status: String,
    pub companyName: String,
    pub companyIds: Vec<String>,
    pub productDescription: String,
    pub productIds: Vec<String>,
    pub productCategoryCpc: i32,
    pub productNameCompany: String,
    pub pcf: Option<serde_json::Value>,
    pub comment: String,
    pub ons: Vec<on>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct on {
    pub specVersion: String,
    pub dataSchema: String,
    pub data: onData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct onData {
    pub mass: f64,
    pub shipmentId: String,
    pub tces: Vec<Tce>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tce {
    pub tceId: String,
    pub prevTceIds: Vec<String>,
    pub hocId: Option<String>,
    pub tocId: Option<String>,
    pub shipmentId: String,
    pub mass: f64,
    pub co2eWTW: Option<serde_json::Value>,
    pub co2eTTW: Option<serde_json::Value>,
    pub transportActivity: Option<serde_json::Value>,
    pub distance: Option<Distance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Distance {
    pub actual: f64,
    pub gcd: Option<serde_json::Value>,
    pub sfd: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TocData {
    pub tocId: String,
    pub certifications: Vec<String>,
    pub description: String,
    pub mode: String,
    pub loadFactor: String,
    pub emptyDistanceFactor: String,
    pub temperatureControl: String,
    pub truckLoadingSequence: String,
    pub airShippingOption: Option<serde_json::Value>,
    pub flightLength: Option<serde_json::Value>,
    pub energyCarriers: Vec<EnergyCarrier>,
    pub co2eIntensityWTW: String,
    pub co2eIntensityTTW: String,
    pub transportActivityUnit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HocData {
    pub hocId: String,
    pub passhubType: String,
    pub energyCarriers: Vec<EnergyCarrier>,
    pub co2eIntensityWTW: String,
    pub co2eIntensityTTW: String,
    pub hubActivityUnit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnergyCarrier {
    pub energyCarrier: String,
    pub distributionEfficiency: String,
    pub energyDensity: String,
    pub unit: String,
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
    // Lese die komplexen product footprint daten
    let product_footprint: ProductFootprintRoot = env::read();
    let mut transport_pcf : f64 = 0.0; 


     let tces : &Vec<Tce> = &product_footprint.productFootprint.ons[0].data.tces; 


     for tce in tces{

        if tce.tocId.is_some() {
           if let Some(distance) = &tce.distance {


    let emission_factor :f64 = emission_factor_toc(&product_footprint.tocData, tce.tocId.clone().unwrap()); 
    let emissions: f64 = tce.mass * emission_factor * distance.actual;  // TODO: Add here a correct emission factor later 
    println!("Emissions from TOC {}: {} g co2e", tce.tceId, emissions);
   transport_pcf += emissions; 
} else {
    println!("Distance is missing");
}

         
        };



        if tce.hocId.is_some(){

            let emission_factor : f64 = emission_factor_hoc(&product_footprint.hocData, tce.hocId.clone().unwrap()); 
            let emissions:f64 = tce.mass * emission_factor; //TODO: Add here the correct emission factor later
            println!("Emissions form HOC {}: {} g co2e", tce.tceId, emissions); 
           transport_pcf += emissions;  
        }
     }






     // TODO: Add later on the indeed verification of the signature of the measurements
  /*  for shipment in shipments {        if verify_signature(&shipment.info) {
            env::log(format!("Shipment {}: GÜLTIG", shipment.shipment_id).as_str());
        
        let data : Value = serde_json::from_str(&shipment.info.activity_data_json).unwrap();    
        let shipment_emissions : f64 = data["co2eWTW"].as_str().unwrap().parse::<f64>().unwrap(); 

        transport_pcf = add_emissions(shipment_emissions, transport_pcf); 
        } else {
            env::log(format!("Shipment {}: UNGÜLTIG", shipment.shipment_id).as_str());

        
        }
    }

    */


    
    fn add_emissions (emissions:f64, pcf_previous :f64) -> f64 {

        let pcf_new :f64 =  pcf_previous +emissions; 
        return pcf_new; 
    }


    fn emission_factor_toc(toc_data:&Vec<TocData>, toc_id: String)-> f64{


       let right_toc_data : &TocData =  toc_data.into_iter().find(|t| { t.tocId == toc_id}).unwrap(); 

       let emission_factor_str : String = right_toc_data.co2eIntensityWTW.clone(); 

       let factor = emission_factor_str.split(" ").next().unwrap().parse::<f64>().unwrap(); 


       return factor; 

    }

        fn emission_factor_hoc(hoc_data:&Vec<HocData>, hoc_id: String)-> f64{


       let right_hoc_data : &HocData =  hoc_data.into_iter().find(|t| {t.hocId == hoc_id}).unwrap(); 

       let emission_factor_str : String = right_hoc_data.co2eIntensityWTW.clone(); 

       let factor = emission_factor_str.split(" ").next().unwrap().parse::<f64>().unwrap(); 


       return factor; 

    }

    





   
    env::log(&format!("Total Emissions {} gram CO2e ", transport_pcf));
    env::log(&format!("Total Emissions {} kg CO2e ", transport_pcf/(1000 as f64)));
    env::commit(&transport_pcf);
}