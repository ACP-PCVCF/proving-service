use crate::sig_verifier::verify_signature;
use base64::engine::general_purpose;
use base64::Engine;
use methods::{
    GUEST_BASELINE_ID, GUEST_BASELINE_PRECOMPILES_ID, GUEST_DUMMY_ID, GUEST_LAZY_PRECOMPILES_ID,
};
use proving_service_core::product_footprint::ProductProof;
use proving_service_core::proof_container::ProofContainer;
use proving_service_core::sig_container::SignatureContainer;
use risc0_zkvm::{sha::Digest, ExecutorEnvBuilder, Receipt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestType {
    Baseline,
    Lazy,
    Dummy,
    Unknown,
}

pub fn get_guest_type(image_id: &Digest) -> GuestType {
    match *image_id {
        id if id == Digest::from(GUEST_BASELINE_ID)
            || id == Digest::from(GUEST_BASELINE_PRECOMPILES_ID) =>
        {
            GuestType::Baseline
        }
        id if id == Digest::from(GUEST_LAZY_PRECOMPILES_ID) => GuestType::Lazy,
        id if id == Digest::from(GUEST_DUMMY_ID) => GuestType::Dummy,
        _ => GuestType::Unknown,
    }
}

/// Guest mode configured via environment variables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfiguredGuestMode {
    Dummy,
    BaselinePrecompiles,
    Baseline,
    LazyPrecompiles,
}

/// Returns the guest mode based on environment variables.
/// Priority: USE_DUMMY_GUEST > USE_BASELINE_PRECOMPILES_GUEST > USE_BASELINE_GUEST > default (LazyPrecompiles)
pub fn get_configured_guest_mode() -> ConfiguredGuestMode {
    let use_dummy = std::env::var("USE_DUMMY_GUEST")
        .unwrap_or_default()
        .to_lowercase()
        == "true";
    let use_baseline_precompiles = std::env::var("USE_BASELINE_PRECOMPILES_GUEST")
        .unwrap_or_default()
        .to_lowercase()
        == "true";
    let use_baseline = std::env::var("USE_BASELINE_GUEST")
        .unwrap_or_default()
        .to_lowercase()
        == "true";

    if use_dummy {
        ConfiguredGuestMode::Dummy
    } else if use_baseline_precompiles {
        ConfiguredGuestMode::BaselinePrecompiles
    } else if use_baseline {
        ConfiguredGuestMode::Baseline
    } else {
        ConfiguredGuestMode::LazyPrecompiles
    }
}

pub fn process_and_write_proofs<'a>(
    proof_vec: &Vec<ProductProof>,
    env_builder: &mut ExecutorEnvBuilder<'a>,
) {
    let mut proof_containers: Vec<ProofContainer> = Vec::new();

    // Check if the proofing document has proofs
    for pcf_proof in proof_vec {
        println!(
            "Found previous proof with productFootprintId: {}",
            pcf_proof.productFootprintId
        );
        // Decode bytes
        let receipt_bytes: Vec<u8> = general_purpose::STANDARD
            .decode(&pcf_proof.proofReceipt)
            .expect(&format!(
                "Error while decoding receipt. Id: {}.",
                pcf_proof.productFootprintId
            ));

        // Deserialize receipt
        let receipt: Receipt = bincode::deserialize(&receipt_bytes).expect(&format!(
            "Error while deserializing receipt. Id: {}.",
            pcf_proof.productFootprintId
        ));

        // Deserialize imageId
        let image_id_vec = hex::decode(&pcf_proof.imageId).expect(&format!(
            "Error decoding hex imageId: {}",
            &pcf_proof.imageId
        ));

        let image_id_bytes: [u8; 32] = image_id_vec
            .try_into()
            .expect("Image ID is not 32 bytes long");

        let image_id = Digest::from(image_id_bytes);

        if let Err(e) = receipt.verify(image_id) {
            eprintln!("Receipt verification failed: {}", e);
            return;
        }
        // Clone Journal
        let journal = receipt.journal.clone();

        // Handle journal decoding and signature verification based on guest type
        match get_guest_type(&image_id) {
            GuestType::Baseline | GuestType::Dummy => {
                // Baseline/Dummy guest: signatures already verified inside zkVM, just decode PCF
                let _journal_output: f64 = match receipt.journal.decode() {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Failed to decode journal: {}", e);
                        return;
                    }
                };
                println!(
                    "Baseline guest proof - skipping host signature verification for productFootprintId: {}",
                    pcf_proof.productFootprintId
                );
            }
            GuestType::Lazy => {
                // Lazy guest: decode journal with signatures and verify on host
                let (_journal_output, serialized_sig_containers): (f64, Vec<u8>) =
                    match receipt.journal.decode() {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("Failed to decode journal: {}", e);
                            return;
                        }
                    };

                let sig_containers =
                    bincode::deserialize::<Vec<SignatureContainer>>(&serialized_sig_containers)
                        .expect("Failed to deserialize signature containers");

                // verify signatures
                for sig_container in &sig_containers {
                    if !verify_signature(
                        &sig_container.commitment,
                        &sig_container.signature,
                        &sig_container.pub_key,
                    ) {
                        eprintln!(
                            "Signature verification failed for proof with productFootprintId: {}",
                            pcf_proof.productFootprintId
                        );
                        return;
                    }
                }
                println!(
                    "Lazy guest proof - verified {} signatures on host for productFootprintId: {}",
                    sig_containers.len(),
                    pcf_proof.productFootprintId
                );
            }
            GuestType::Unknown => {
                eprintln!(
                    "Unknown guest image ID for proof with productFootprintId: {}",
                    pcf_proof.productFootprintId
                );
                return;
            }
        }

        // Create ProofContainer
        let proof_container = ProofContainer { image_id, journal };

        // Add assumption
        env_builder.add_assumption(receipt);

        // Append vector
        proof_containers.push(proof_container);
    }

    // Serialize proof containers
    let serialized_proof_containers =
        bincode::serialize(&proof_containers).expect("Failed to serialize proof_containers");

    // Write to env_builder
    env_builder
        .write(&serialized_proof_containers)
        .expect("Error while writing ProofContainers to Builder.");
}
