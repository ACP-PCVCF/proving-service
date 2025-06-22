use proving_service_core::product_footprint::ProofExtension;
use risc0_zkvm::{ Receipt, ExecutorEnvBuilder, sha::Digest };
use base64::engine::general_purpose;
use base64::Engine;
use proving_service_core::proof_container::ProofContainer;

pub fn process_and_write_proofs<'a>(
    proof_extension_opt: &Option<ProofExtension>,
    env_builder: &mut ExecutorEnvBuilder<'a>
) {
    let mut proof_containers: Vec<ProofContainer> = Vec::new();

    // Check if the proofing document has proofs
    if let Some(proof_extension) = &proof_extension_opt {
        // Iterate proofs
        for pcf_proof in &proof_extension.data.pcfProofs {
            // Decode bytes
            let receipt_bytes: Vec<u8> = general_purpose::STANDARD
                .decode(&pcf_proof.proofReceipt)
                .expect(
                    &format!("Error while decoding receipt. Id: {}.", pcf_proof.productFootprintId)
                );

            // Deserialize receipt
            let receipt: Receipt = bincode
                ::deserialize(&receipt_bytes)
                .expect(
                    &format!(
                        "Error while deserializing receipt. Id: {}.",
                        pcf_proof.productFootprintId
                    )
                );

            // Deserialize imageId
            let image_id = Digest::new(pcf_proof.imageId);

            // Clone Journal
            let journal = receipt.journal.clone();

            // Create ProofContainer
            let proof_container = ProofContainer {
                image_id,
                journal,
            };

            // Add assumption
            env_builder.add_assumption(receipt);

            // Append vector
            proof_containers.push(proof_container);

            // println!(
            //     "ProofContainer created for productFootprintId: {}",
            //     pcf_proof.productFootprintId
            // );
        }

        // Serialize proof containers
        let serialized_proof_containers = bincode
            ::serialize(&proof_containers)
            .expect("Failed to serialize proof_containers");

        // Write to env_builder
        env_builder
            .write(&serialized_proof_containers)
            .expect("Error while writing ProofContainers to Builder.");
    }
}
