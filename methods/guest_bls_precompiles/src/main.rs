extern crate alloc;
use alloc::{format, string::String, vec::Vec};
use base64::{engine::general_purpose, Engine as _};
use blst::min_pk::{PublicKey, Signature};
use blst::BLST_ERROR;
use proving_service_core::hoc_toc_data::*;
use proving_service_core::proof_container::ProofContainer;
use proving_service_core::proofing_document::*;
use risc0_zkvm::guest::env;
use risc0_zkvm::guest::sha::rust_crypto::Sha256;
use risc0_zkvm::guest::sha::Impl as Sha256Impl;
use risc0_zkvm::sha::Digest;
use risc0_zkvm::Journal;
use sha2::digest::Update;
use sha2::Digest as Sha2DigestTrait;

// Domain Separation Tag for BLS signatures.
const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

fn hash(data: &str) -> String {
    let mut hasher = Sha256::<Sha256Impl>::new();
    Update::update(&mut hasher, data.as_bytes());
    let computed_hash = hasher.finalize();
    general_purpose::STANDARD.encode(&computed_hash)
}

fn process_proof_containers(proof_containers: &[ProofContainer], initial_pcf: f64) -> f64 {
    let mut pcf = initial_pcf;

    for container in proof_containers {
        let image_id: Digest = container.image_id.clone();
        let journal: Journal = container.journal.clone();

        env::verify(image_id.clone(), journal.bytes.as_slice()).unwrap();
        env::log(&format!("Verified proof with image ID: {}", image_id));

        let prev_pcf: f64 = journal.decode().expect("Failed to decode pcf");
        pcf += prev_pcf;
    }

    pcf
}

fn emission_factor_toc(toc_data: &Vec<TocData>, toc_id: String) -> f64 {
    let toc = toc_data.iter().find(|t| t.tocId == toc_id).unwrap();
    toc.co2eIntensityWTW
        .split(' ')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap()
}

fn emission_factor_hoc(hoc_data: &Vec<HocData>, hoc_id: String) -> f64 {
    let hoc = hoc_data.iter().find(|t| t.hocId == hoc_id).unwrap();
    hoc.co2eIntensityWTW
        .split(' ')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap()
}

fn main() {
    env::log("Guest BLS: Starting...");
    let mut transport_pcf: f64 = 0.0;

    // Read inputs
    let document: ProofingDocument = env::read();

    // Aggregated BLS signature
    let agg_sig_bytes: Vec<u8> = env::read();

    let serialized_proof_containers: Vec<u8> = env::read();
    let proof_containers: Vec<ProofContainer> = bincode::deserialize(&serialized_proof_containers)
        .expect("Failed to deserialize proof_containers");

    // Verify previous proofs
    transport_pcf = process_proof_containers(&proof_containers, transport_pcf);

    // Collect messages and public keys while processing TCEs
    // These will be verified against the aggregated signature
    let mut messages: Vec<Vec<u8>> = Vec::new();
    let mut pubkeys: Vec<PublicKey> = Vec::new();

    let tces = &document.productFootprint.extensions[0].data.tces;

    for tce in tces {
        if let Some(toc_id) = &tce.tocId {
            if let Some(distance) = &tce.distance {
                let emission_factor = emission_factor_toc(&document.tocData, toc_id.clone());
                let emissions = tce.mass * emission_factor * distance.actual;

                if let Some(sensor_data_list) = &document.signedSensorData {
                    for sensor_data in sensor_data_list {
                        if sensor_data.tceId == tce.tceId {
                            // Verify hash commitment
                            let concat = format!(
                                "{}{}",
                                serde_json::to_string(&sensor_data.sensorData).unwrap(),
                                sensor_data.salt
                            );
                            let computed_hash = hash(&concat);
                            assert!(
                                computed_hash == sensor_data.commitment,
                                "Commitment does not match"
                            );

                            // Collect for batch verification
                            messages.push(sensor_data.commitment.as_bytes().to_vec());

                            // Decode base64-encoded BLS public key
                            let pk_bytes = general_purpose::STANDARD
                                .decode(&sensor_data.sensorkey)
                                .expect("Invalid base64 public key");
                            let pk =
                                PublicKey::from_bytes(&pk_bytes).expect("Invalid BLS public key");
                            pubkeys.push(pk);
                        }
                    }
                }

                transport_pcf += emissions;
            }
        }

        if let Some(hoc_id) = &tce.hocId {
            let emission_factor = emission_factor_hoc(&document.hocData, hoc_id.clone());
            transport_pcf += tce.mass * emission_factor;
        }
    }

    // BLS aggregate verification
    // Verifies that each public key signed its corresponding message.
    if !messages.is_empty() {
        env::log(&format!("Verifying {} signatures", messages.len()));

        // Aggregated signature
        let agg_sig = Signature::from_bytes(&agg_sig_bytes).expect("Invalid aggregate signature");

        let msgs_refs: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
        let pks_refs: Vec<&PublicKey> = pubkeys.iter().collect();

        let result = agg_sig.aggregate_verify(
            true,       // Verify signature is in the correct subgroup
            &msgs_refs, // All messages (commitments) that were signed
            DST,        // Domain separation tag - must match what signers used
            &pks_refs,  // All public keys corresponding to each message
            true,       // Verify public keys are in the correct subgroup
        );

        assert!(
            result == BLST_ERROR::BLST_SUCCESS,
            "BLS aggregate signature verification failed"
        );

        env::log("All signatures verified");
    }

    env::log(&format!("Total emissions: {} kg CO2e", transport_pcf));
    env::commit(&transport_pcf);
}
