extern crate alloc;
use bincode;
use alloc::{ vec::Vec, string::String, format };
use risc0_zkvm::guest::env;
use risc0_zkvm::Receipt;
use risc0_zkvm::Journal;
use risc0_zkvm::sha::Digest;
use rsa::{ RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey };
use rsa::pkcs1v15::Pkcs1v15Sign;
use sha2::{ Sha256, Digest as Sha2DigestTrait };
use base64::{ engine::general_purpose, Engine as _ };
use std::{ * };
use serde_json;
use proving_service_core::proofing_document::*;
use proving_service_core::hoc_toc_data::*;
use proving_service_core::product_footprint::*;
use rsa::pkcs8::AssociatedOid;
use pkcs1::ObjectIdentifier;
use sha2::digest::{
    Digest as DigestTrait,
    OutputSizeUser,
    Reset,
    FixedOutputReset,
    generic_array::GenericArray,
    FixedOutput,
    Update,
};

//risc0_zkvm::guest::entry!(main);

fn sum_emissions(val: f32, emissions: f32) -> f32 {
    return val + emissions;
}

fn sum_mass(mass: f32, total_mass: f32) -> f32 {
    return total_mass + mass;
}

fn verify_signature(info: &TceSensorData) -> bool {
    let payload = &info.sensorData;
    let signature_b64 = &info.signedSensorData;
    let public_key_pem = &info.sensorkey;

    env::log(format!("Payload: {}", payload).as_str());
    env::log(format!("Signature: {}", signature_b64).as_str());
    env::log(format!("Public Key PEM: {}", public_key_pem).as_str());

    let public_key = match RsaPublicKey::from_public_key_pem(public_key_pem) {
        Ok(pk) => pk,
        Err(e) => {
            env::log(format!("Error loading public key (SPKI expected): {:?}", e).as_str());
            match RsaPublicKey::from_pkcs1_pem(public_key_pem) {
                Ok(pk_fallback) => {
                    env::log("Warning: Public key loaded as PKCS#1, SPKI is preferred.");
                    pk_fallback
                }
                Err(e_fallback) => {
                    env::log(
                        format!(
                            "Error loading the public key even as PKCS#1: {:?}",
                            e_fallback
                        ).as_str()
                    );
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
            env::log(format!("Error decoding signature: {:?}", e).as_str());
            return false;
        }
    };

    let padding = Pkcs1v15Sign::new::<Sha256WithOid>();
    match public_key.verify(padding, &digest_val, &signature) {
        Ok(_) => {
            env::log("Signature is valid.");
            true
        }
        Err(e) => {
            env::log(format!("Verification error: {:?}", e).as_str());
            false
        }
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

fn main() {
    // Lese die komplexen product footprint daten
    //let start = env::cycle_count();
    let product_footprint: ProofingDocument = env::read();
    let serialized_image_id_opt_bytes: Vec<u8> = env::read();
    let image_id_opt: Option<Digest> = bincode::deserialize(&serialized_image_id_opt_bytes)
        .expect("Failed to deserialize image_id_opt on guest");

    let serialized_journal_opt_bytes: Vec<u8> = env::read();
    let journal_opt: Option<Journal> = bincode::deserialize(&serialized_journal_opt_bytes)
        .expect("Failed to deserialize journal_opt on guest");
    let mut transport_pcf: f64 = 0.0;

    let ileap_extension: &Extension = &product_footprint.productFootprint.extensions[0];

    image_id_opt.and_then(|image_id| {
        journal_opt.map(|journal| {
            env::verify(image_id, journal.bytes.as_slice()).unwrap();
            env::log(&format!("Guest: Image ID verified successfully: {}", image_id));
        })
    });
/* 
    if let Some(proof_extension) = &product_footprint.proof {
        let receipt_bytes: Vec<u8> = general_purpose::STANDARD
            .decode(&proof_extension.data.pcfProofs[0].proofReceipt)
            .expect("Guest: Fehler beim Deserialisieren des Receipts.");
        env::log(&format!("Guest: Receipt erfolgreich deserialisiert."));

        let receipt: Receipt = bincode
            ::deserialize(&receipt_bytes)
            .expect("Failed to deserialize inner receipt bytes");

        let inner_image_id: Digest = Digest::new(proof_extension.data.pcfProofs[0].imageId);

        match receipt.verify(inner_image_id.clone()) {
            Ok(()) => {
                env::log(&format!("Guest: Innerer Proof erfolgreich verifiziert!"));
                let journal: Journal = receipt.journal;
                env::log(
                    &format!(
                        "Guest: Journal aus innerem Proof gelesen (Länge: {} Bytes).",
                        journal.bytes.len()
                    )
                );

                let inner_program_output: f64 = journal
                    .decode()
                    .expect("Guest: Fehler beim Dekodieren des Journals des inneren Proofs.");
                env::log(
                    &format!("Guest: Inhalt des inneren Journals: '{}'", inner_program_output)
                );

                env::commit(
                    &format!("Innerer Proof erfolgreich verifiziert. Ausgabe: {}", inner_program_output)
                );
                env::log(&format!("Guest: Verifizierungsstatus ins Journal geschrieben."));
            }
            Err(e) => {
                env::log(
                    &format!("Guest: Fehler bei der Verifizierung des inneren Proofs: {:?}", e)
                );
                panic!("Innerer Proof konnte nicht verifiziert werden!");
            }
        }
    } */

    let tces: &Vec<TCE> = &ileap_extension.data.tces;
    let ssd: &Vec<TceSensorData> = &product_footprint.signedSensorData
        .as_ref()
        .expect("No signedSensorData found");

    for tce in tces {
        if tce.tocId.is_some() {
            if let Some(distance) = &tce.distance {
                let emission_factor: f64 = emission_factor_toc(
                    &product_footprint.tocData,
                    tce.tocId.clone().unwrap()
                );

                /*
                let found_tsd_iter = ssd.iter().find(|obj| obj.tceId == tce.tceId);
                if let Some(tsd) = found_tsd_iter {
                    if verify_signature(tsd) {
                        env::log(
                            format!(
                                "Verification for sensor data related to TCE '{}', sensor key snippet '{}...': SUCCESS",
                                tsd.tceId,
                                tsd.sensorkey.chars().take(10).collect::<String>()
                            ).as_str()
                        );
                    } else {
                        env::log(
                            format!(
                                "Verification for sensor data related to TCE '{}', sensor key snippet '{}...': INVALID",
                                tsd.tceId,
                                tsd.sensorkey.chars().take(10).collect::<String>()
                            ).as_str()
                        );
                        env::exit(1);
                    }
                }
                */

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
    env::commit(&transport_pcf);
    env::log(&format!("End of guest programm. Proof can take a while..."));
    //let end = env::cycle_count();
}
