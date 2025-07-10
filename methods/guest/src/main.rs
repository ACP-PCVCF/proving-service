extern crate alloc;
use bincode;
use alloc::{ vec::Vec, string::String, format };
use proving_service_core::proof_container::ProofContainer;
use proving_service_core::sig_container::SignatureContainer;
use risc0_zkvm::guest::env;
use risc0_zkvm::Journal;
use risc0_zkvm::sha::Digest;
use sha2::{ Sha256, Digest as Sha2DigestTrait };
use base64::{ engine::general_purpose, Engine as _ };
use std::{ * };
use proving_service_core::proofing_document::*;
use proving_service_core::hoc_toc_data::*;
use proving_service_core::product_footprint::*;
use sha2::digest::{Update};

fn hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    Update::update(&mut hasher, data.as_bytes());
    let computed_hash = hasher.finalize();
    let computed_hash_b64 = general_purpose::STANDARD.encode(computed_hash);
    return computed_hash_b64
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
    // Initialize
    env::log("Guest: Starting the guest program...");
    let mut transport_pcf: f64 = 0.0;

    // Read inputs
    env::log("Guest: Reading Inputs...");
    let mut sig_containers: Vec<SignatureContainer> = Vec::new();
    let product_footprint: ProofingDocument = env::read();
    let serialized_proof_containers: Vec<u8> = env::read();
    let proof_containers: Vec<ProofContainer> = bincode::deserialize(&serialized_proof_containers)
        .expect("Guest: Failed to deserialize proof_containers");

    // Verify previous proofs and add pcf value 
    transport_pcf = process_proof_containers(&proof_containers, transport_pcf);

    let ileap_extension: &Extension = &product_footprint.productFootprint.extensions[0];

    let tces: &Vec<TCE> = &ileap_extension.data.tces;

    for tce in tces {
        if tce.tocId.is_some() {
            if let Some(distance) = &tce.distance {
                let emission_factor: f64 = emission_factor_toc(
                    &product_footprint.tocData,
                    tce.tocId.clone().unwrap()
                );       

                let emissions: f64 = tce.mass * emission_factor * distance.actual;

                if let Some(signed_sensor_data_list) = &product_footprint.signedSensorData {
                    for signed_sensor_data in signed_sensor_data_list {
                        if signed_sensor_data.tceId == tce.tceId {
                            let concat = format!("{}{}", serde_json::to_string(&signed_sensor_data.sensorData).unwrap(), signed_sensor_data.salt);
                            assert!(hash(&concat) == signed_sensor_data.commitment, "Commitment does not match the hash of sensor data and salt");
                            sig_containers.push(SignatureContainer {
                                commitment: signed_sensor_data.commitment.clone(),
                                signature: signed_sensor_data.signedSensorData.clone(),
                                pub_key: signed_sensor_data.sensorkey.clone(),
                            });
                        }
                    }
                }

                transport_pcf += emissions;
            } else {
                env::log("Distance is missing"); 
            }
        }

        if tce.hocId.is_some() {
            let emission_factor: f64 = emission_factor_hoc(
                &product_footprint.hocData,
                tce.hocId.clone().unwrap()
            );
            let emissions: f64 = tce.mass * emission_factor;
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
    let serialized_sig_containers: Vec<u8> = bincode::serialize(&sig_containers)
        .expect("Failed to serialize sig_containers");
    env::commit(&serialized_sig_containers);
}
