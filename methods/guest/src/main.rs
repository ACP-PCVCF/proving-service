extern crate alloc;
use bincode;
use alloc::{ vec::Vec, string::String, format };
use proving_service_core::proof_container::ProofContainer;
use risc0_zkvm::guest::env;
use risc0_zkvm::Journal;
use risc0_zkvm::sha::Digest;
use rsa::{ RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey };
use rsa::pkcs1v15::Pkcs1v15Sign;
use sha2::{ Sha256, Digest as Sha2DigestTrait };
use base64::{ engine::general_purpose, Engine as _ };
use std::{ * };
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

fn process_proof_containers(
    proof_containers: &[ProofContainer],
    initial_transport_pcf: f64,
) -> f64 {
    let mut current_transport_pcf = initial_transport_pcf;

    for proof_container in proof_containers {
        let image_id: Digest = proof_container.image_id.clone();
        let journal: Journal = proof_container.journal.clone();

        env::verify(image_id.clone(), journal.bytes.as_slice()).unwrap();
        env::log(&format!("Guest: Image ID verified successfully: {}", image_id));

        let pcf: f64 = journal.decode().expect("Failed to decode journal");
        env::log(&format!("Guest: PCF value from previous proof: {}", pcf));
        current_transport_pcf = pcf + current_transport_pcf;
    }

    current_transport_pcf
}

fn main() {
    let start = env::cycle_count();

    // Initialize
    env::log("Guest: Starting the guest program...");
    let mut transport_pcf: f64 = 0.0;

    // Read inputs
    env::log("Guest: Reading Inputs...");
    let product_footprint: ProofingDocument = env::read();
    let serialized_proof_containers: Vec<u8> = env::read();
    let proof_containers: Vec<ProofContainer> = bincode::deserialize(&serialized_proof_containers)
        .expect("Guest: Failed to deserialize proof_containers");

    // Verify previous proofs and add pcf value 
    transport_pcf = process_proof_containers(&proof_containers, transport_pcf);

    let ileap_extension: &Extension = &product_footprint.productFootprint.extensions[0];

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
    let end = env::cycle_count();
    env::log(&format!("End of guest programm. Cycles: {}", end - start));
}
