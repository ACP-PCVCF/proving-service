extern crate alloc;
use alloc::vec::Vec;
use proving_service_core::proof_container::ProofContainer;
use proving_service_core::proofing_document::ProofingDocument;
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::Digest;
use risc0_zkvm::Journal;

fn main() {
    env::log("Guest Dummy: Starting minimal proof verification...");

    // Read inputs
    let _product_footprint: ProofingDocument = env::read();
    let serialized_proof_containers: Vec<u8> = env::read();
    let proof_containers: Vec<ProofContainer> = bincode::deserialize(&serialized_proof_containers)
        .expect("Guest Dummy: Failed to deserialize proof_containers");

    let mut verified_count: u64 = 0;

    // Verify each proof
    for proof_container in &proof_containers {
        let image_id: Digest = proof_container.image_id.clone();
        let journal: Journal = proof_container.journal.clone();

        env::verify(image_id.clone(), journal.bytes.as_slice()).unwrap();
        env::log(&format!(
            "Guest Dummy: Verified proof with image ID: {}, journal size: {} bytes",
            image_id,
            journal.bytes.len()
        ));

        verified_count += 1;
    }

    env::log(&format!(
        "Guest Dummy: Successfully verified {} proof(s)",
        verified_count
    ));

    env::commit(&verified_count);
}
