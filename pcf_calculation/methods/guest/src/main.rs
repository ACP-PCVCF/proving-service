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
use metrics::metrics::guest_metrics::GuestMetrics;

risc0_zkvm::guest::entry!(main);

/*#[derive(Deserialize, Serialize)]
struct ShipmentInfo {
    #[serde(rename = "sensorData")]
    sensor_data: String,
    #[serde(rename = "signedSensorData")]
    signed_sensor_data: String,
    sensorkey: String,
}

#[derive(Deserialize, Serialize)]
struct Shipment {
    //shipment_id: String,
    info: ShipmentInfo,
}*/

#[derive(Deserialize, Serialize)]
struct Distance {
    actual: f64, // oder f32, je nach Genauigkeit. JSON 'number' wird oft zu f64.
    gcd: Option<f64>, // Da es 'null' sein kann
    sfd: Option<f64>, // Da es 'null' sein kann
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
}

#[derive(Deserialize, Serialize)]
struct Activity {
    process_id: String,
    unit: String,
    consumption: u32,
    e_type: String,
}

/*#[derive(Deserialize, Serialize)]
struct CombinedInput {
    activities: Vec<Activity>,
    shipments: Vec<Shipment>,
}*/
#[derive(Deserialize, Serialize)]
struct CombinedInput {
    activities: Vec<Activity>,
    //shipments: Vec<Shipment>,
    signatures: Vec<SignedSensorData>,
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

//fn verify_signature(info: &ShipmentInfo) -> bool {
fn verify_signature(info: &SignedSensorData) -> bool {
    /*let payload_string = match serde_json::to_string(&info.sensor_data) {
        Ok(s) => s,
        Err(e) => {
            env::log(format!("Fehler beim Serialisieren von sensor_data_payload zu String: {:?}", e).as_str());
            return false;
        }
    };
    let payload = payload_string.as_bytes();*/

    let payload = &info.sensor_data;
    let signature_b64 = &info.signed_sensor_data;
    let public_key_pem = &info.sensorkey;
    //env::log(format!("Payload (Bytes): {:?}", payload).as_str());
    env::log(format!("Payload: {}", payload).as_str());
    env::log(format!("Signature: {}", signature_b64).as_str());
    env::log(format!("Public Key PEM: {}", public_key_pem).as_str());



    let public_key = match RsaPublicKey::from_public_key_pem(public_key_pem) {
        Ok(pk) => pk,
        Err(e) => {
            env::log(format!("Fehler beim Laden des Public Keys (SPKI erwartet): {:?}", e).as_str());
            match RsaPublicKey::from_pkcs1_pem(public_key_pem) {
                Ok(pk_fallback) => {
                    env::log("Warnung: Public Key wurde als PKCS#1 geladen, SPKI wird bevorzugt.");
                    pk_fallback
                },
                Err(e_fallback) => {
                    env::log(format!("Fehler beim Laden des Public Keys auch als PKCS#1: {:?}", e_fallback).as_str());
                    return false;
                }
            }
        }
    };

    let mut hasher = Sha256::new();
    Update::update(&mut hasher, payload.as_bytes());
    //Update::update(&mut hasher, payload);
    let digest_val = hasher.finalize();

    let signature = match general_purpose::STANDARD.decode(signature_b64) {
        Ok(sig) => sig,
        Err(e) => {
            env::log(format!("Fehler beim Dekodieren der Signatur: {:?}", e).as_str());
            return false;
        }
    };

    let padding = Pkcs1v15Sign::new::<Sha256WithOid>();
    match public_key.verify(padding, &digest_val, &signature) {
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
    // Metriken initialisieren
    let mut guest_metrics = GuestMetrics::new();

    let input: CombinedInput = env::read();
    let valid_activities: Vec<Activity> = input.activities; 

    // Start der Zykluszählung
    guest_metrics.start_riscv_cyc_count();
        
    for signature in input.signatures {
        if verify_signature(&signature) {
            //env::log(format!("Shipment {}: GÜLTIG", shipment.shipment_id).as_str());
            env::log(format!("Erfolg").as_str());
        } else {
            //env::log(format!("Shipment {}: UNGÜLTIG", shipment.shipment_id).as_str());
            env::log(format!("Signatur: UNGÜLTIG").as_str());
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
    //env::commit(&pcf_total);

        // Ende der Zykluszählung
    guest_metrics.end_riscv_cyc_count();

    // Ergebnis und Metriken an den Host committen
    env::commit(&(&pcf_total, guest_metrics));
}
