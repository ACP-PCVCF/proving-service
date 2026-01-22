extern crate alloc;
use alloc::{format, string::String, vec::Vec};
use base64::{engine::general_purpose, Engine as _};
use bincode;
use proving_service_core::hoc_toc_data::*;
use proving_service_core::product_footprint::*;
use proving_service_core::proof_container::ProofContainer;
use proving_service_core::proofing_document::*;
use risc0_zkvm::guest::env;
use risc0_zkvm::guest::sha::rust_crypto::Sha256;
use risc0_zkvm::guest::sha::Impl as Sha256Impl;
use risc0_zkvm::sha::Digest;
use risc0_zkvm::Journal;
use rsa::{pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey, RsaPublicKey};
use sha2::digest::Update;
use sha2::Digest as Sha2DigestTrait;
use std::*;

fn hash(data: &str) -> String {
    let mut hasher = Sha256::<Sha256Impl>::new();
    Update::update(&mut hasher, data.as_bytes());
    let computed_hash = hasher.finalize();
    let computed_hash_b64 = general_purpose::STANDARD.encode(&computed_hash);
    return computed_hash_b64;
}

fn verify_signature_in_guest(commitment: &str, signed_sensor_data: &str, sensorkey: &str) -> bool {
    let public_key = match RsaPublicKey::from_public_key_pem(sensorkey) {
        Ok(pk) => pk,
        Err(_) => {
            // Fallback to PKCS#1
            match RsaPublicKey::from_pkcs1_pem(sensorkey) {
                Ok(pk_fallback) => pk_fallback,
                Err(_) => {
                    env::log("Guest Baseline: Failed to parse public key");
                    return false;
                }
            }
        }
    };

    // Hash the commitment
    let mut hasher = Sha256::<Sha256Impl>::new();
    Update::update(&mut hasher, commitment.as_bytes());
    let digest_val = hasher.finalize();

    // Decode signature from base64
    let signature = match general_purpose::STANDARD.decode(signed_sensor_data) {
        Ok(sig) => sig,
        Err(_) => {
            env::log("Guest Baseline: Failed to decode signature");
            return false;
        }
    };

    // Verify signature
    let padding = rsa::Pkcs1v15Sign::new::<Sha256<Sha256Impl>>();
    match public_key.verify(padding, &digest_val, &signature) {
        Ok(_) => {
            env::log("Guest Baseline: Signature verified successfully");
            true
        }
        Err(_) => {
            env::log("Guest Baseline: Signature verification failed");
            false
        }
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
        env::log(&format!(
            "Guest Baseline: Image ID verified successfully: {}",
            image_id
        ));

        let pcf: f64 = journal.decode().expect("Failed to decode journal");
        env::log(&format!(
            "Guest Baseline: PCF value from previous proof: {}",
            pcf
        ));

        current_transport_pcf = pcf + current_transport_pcf;
    }

    current_transport_pcf
}

fn main() {
    // Initialize
    env::log("Guest Baseline: Starting with signature verification...");
    let mut transport_pcf: f64 = 0.0;

    // Read inputs
    env::log("Guest Baseline: Reading Inputs...");
    let product_footprint: ProofingDocument = env::read();
    let serialized_proof_containers: Vec<u8> = env::read();
    let proof_containers: Vec<ProofContainer> = bincode::deserialize(&serialized_proof_containers)
        .expect("Guest Baseline: Failed to deserialize proof_containers");

    // Verify previous proofs and add pcf value
    transport_pcf = process_proof_containers(&proof_containers, transport_pcf);

    let ileap_extension: &Extension = &product_footprint.productFootprint.extensions[0];

    let tces: &Vec<TCE> = &ileap_extension.data.tces;

    for tce in tces {
        if tce.tocId.is_some() {
            if let Some(distance) = &tce.distance {
                let emission_factor: f64 =
                    emission_factor_toc(&product_footprint.tocData, tce.tocId.clone().unwrap());

                let emissions: f64 = tce.mass * emission_factor * distance.actual;

                // BASELINE: Verify signatures immediately
                if let Some(signed_sensor_data_list) = &product_footprint.signedSensorData {
                    for signed_sensor_data in signed_sensor_data_list {
                        if signed_sensor_data.tceId == tce.tceId {
                            let concat = format!(
                                "{}{}",
                                serde_json::to_string(&signed_sensor_data.sensorData).unwrap(),
                                signed_sensor_data.salt
                            );
                            let computed_hash = hash(&concat);
                            assert!(
                                computed_hash == signed_sensor_data.commitment,
                                "Commitment does not match the hash of sensor data and salt"
                            );

                            // BASELINE: Verify signature immediately (not lazy)
                            env::log(&format!(
                                "Guest Baseline: Verifying signature for TCE {}",
                                tce.tceId
                            ));
                            let verified = verify_signature_in_guest(
                                &signed_sensor_data.commitment,
                                &signed_sensor_data.signedSensorData,
                                &signed_sensor_data.sensorkey,
                            );
                            assert!(verified, "Guest Baseline: Signature verification failed for current document");
                        }
                    }
                }

                transport_pcf += emissions;
            } else {
                env::log("Distance is missing");
            }
        }

        if tce.hocId.is_some() {
            let emission_factor: f64 =
                emission_factor_hoc(&product_footprint.hocData, tce.hocId.clone().unwrap());
            let emissions: f64 = tce.mass * emission_factor;
            transport_pcf += emissions;
        }
    }

    fn emission_factor_toc(toc_data: &Vec<TocData>, toc_id: String) -> f64 {
        let right_toc_data: &TocData = toc_data.into_iter().find(|t| t.tocId == toc_id).unwrap();

        let emission_factor_str: String = right_toc_data.co2eIntensityWTW.clone();

        let factor = emission_factor_str
            .split(" ")
            .next()
            .unwrap()
            .parse::<f64>()
            .unwrap();

        return factor;
    }

    fn emission_factor_hoc(hoc_data: &Vec<HocData>, hoc_id: String) -> f64 {
        let right_hoc_data: &HocData = hoc_data.into_iter().find(|t| t.hocId == hoc_id).unwrap();

        let emission_factor_str: String = right_hoc_data.co2eIntensityWTW.clone();

        let factor = emission_factor_str
            .split(" ")
            .next()
            .unwrap()
            .parse::<f64>()
            .unwrap();

        return factor;
    }

    env::log(&format!(
        "Guest Baseline: Total Emissions {} kg CO2e",
        transport_pcf
    ));

    env::commit(&transport_pcf);
}
