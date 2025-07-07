use proving_service_core::product_footprint::{ProductProof, ProofExtension};
use proving_service_core::proofing_document::TceSensorData;
use proving_service_core::sig_container::SignatureContainer;
use risc0_zkvm::InnerReceipt;
use risc0_zkvm::{ Receipt, ExecutorEnvBuilder, sha::Digest };
use base64::engine::general_purpose;
use base64::Engine;
use proving_service_core::proof_container::ProofContainer;
use crate::sig_verifier::verify_signature;
use anyhow;

pub fn process_and_write_proofs<'a>(
    proof_vec: &Vec<ProductProof>,
    env_builder: &mut ExecutorEnvBuilder<'a>
) {
    let mut proof_containers: Vec<ProofContainer> = Vec::new();

    // Check if the proofing document has proofs
    // if let Some(proof_extension) = &proof_extension_opt {
        // Iterate proofs
        for pcf_proof in proof_vec {
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
            let image_id_vec = hex::decode(&pcf_proof.imageId)
                .expect(&format!("Error decoding hex imageId: {}", &pcf_proof.imageId));

            let image_id_bytes: [u8; 32] = image_id_vec.try_into().expect("Image ID is not 32 bytes long");

            let image_id = Digest::from(image_id_bytes);
            
            // debug
            // let receipt = pcf_proof.receipt.clone();
            // let image_id = Digest::from(pcf_proof.image_id);

    // println!("WSL/Debian Journal (HEX): {}", hex::encode(&receipt.journal));
    // match &receipt.inner {
    //     InnerReceipt::Composite(composite_receipt) => {
    //         for (i, segment) in composite_receipt.segments.iter().enumerate() {
    //             let mut seal_bytes = Vec::new();
    //             // Die `seal` hier ist ein &[u32], das in Bytes umgewandelt werden muss.
    //             // RISC-V ist Little-Endian, daher `to_le_bytes()`.
    //             for &word in segment.seal.iter() {
    //                 seal_bytes.extend_from_slice(&word.to_le_bytes());
    //             }
    //             println!("WSL/Debian Segment {} Seal (HEX): {}", i, hex::encode(&seal_bytes));
    //         }
    //     }
    //     InnerReceipt::Succinct(verifier_receipt) => {
    //         // Dies ist der Fall für ein einzelnes (nicht-komposites) Receipt.
    //         // Falls du eine ältere Version hast oder nicht-komposite Proofs erstellst.
    //         let mut seal_bytes = Vec::new();
    //         for &word in verifier_receipt.seal.iter() {
    //             seal_bytes.extend_from_slice(&word.to_le_bytes());
    //         }
    //         println!("WSL/Debian Succinct Seal (HEX): {}", hex::encode(&seal_bytes));
    //     }
    //     InnerReceipt::Groth16(groth16_receipt) => todo!(),
    //     InnerReceipt::Fake(fake_receipt) => todo!(),
    //     _ => todo!(),
    // }
    // println!("WSL/Debian Image ID (HEX): {}", hex::encode(bytemuck::cast_slice(&image_id_bytes)));


            if let Err(e) = receipt.verify(image_id) {
                eprintln!("Host: Receipt verification failed: {}", e);
                return;
            }
            // Clone Journal
            let journal = receipt.journal.clone();

            // Get journal data
                        let (journal_output, serialized_sig_containers): (f64, Vec<u8>) = match receipt.journal.decode() {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Host: Failed to decode journal: {}", e);
                    return;
                }
            };    

            let sig_containers = bincode::deserialize::<Vec<SignatureContainer>>(
                &serialized_sig_containers
            ).expect("Failed to deserialize signature containers");

            // verify signatures
            for sig_container in &sig_containers {
                if !verify_signature(
                    &sig_container.commitment,
                    &sig_container.signature,
                    &sig_container.pub_key,
                ) {
                    eprintln!("Signature verification failed");
                    continue;
                }
            }

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
    // }

    // Serialize proof containers
    let serialized_proof_containers = bincode
        ::serialize(&proof_containers)
        .expect("Failed to serialize proof_containers");

    // Write to env_builder
    env_builder
        .write(&serialized_proof_containers)
        .expect("Error while writing ProofContainers to Builder.");
}

// pub fn process_and_write_signatures<'a>(
//     signed_sensor_data_vec_opt: &Option<Vec<TceSensorData>>,
//     env_builder: &mut ExecutorEnvBuilder<'a>
// ) {
//     let mut sig_containers: Vec<SignatureContainer> = Vec::new();

//     // Check if the proofing document has signatures
//     if let Some(signed_sensor_data_vec) = &signed_sensor_data_vec_opt {
//         // Iterate
//         for signed_sensor_data in signed_sensor_data_vec {
//             if
//                 verify_signature(
//                     &signed_sensor_data.commitment,
//                     &signed_sensor_data.signedSensorData,
//                     &signed_sensor_data.sensorkey
//                 )
//             {
//                 // Create SignatureContainer
//                 let sig_container = SignatureContainer {
//                     tceId: signed_sensor_data.tceId.clone(),
//                     commitment: signed_sensor_data.commitment.clone(),
//                     salt: signed_sensor_data.salt.clone(),
//                     sensorData: serde_json::to_string(&signed_sensor_data.sensorData)
//                         .expect("Failed to serialize sensorData"),
//                 };

//                 sig_containers.push(sig_container);
//             } else {
//                 eprintln!("Signatur: UNGÜLTIG");
//                 continue;
//             }
//         }

//         // Serialize proof containers
//         let serialized_sig_containers = bincode
//             ::serialize(&sig_containers)
//             .expect("Failed to serialize proof_containers");

//         // Write to env_builder
//         env_builder
//             .write(&serialized_sig_containers)
//             .expect("Error while writing ProofContainers to Builder.");
//     }
// }

// pub fn process_and_write_signatures2<'a>(
//     signed_sensor_data_vec_opt: &Option<Vec<TceSensorData>>,
//     env_builder: &mut ExecutorEnvBuilder<'a>
// ) {
//     let mut sig_containers: Vec<SignatureContainer> = Vec::new();

//     // Check if the proofing document has signatures
//     if let Some(signed_sensor_data_vec) = &signed_sensor_data_vec_opt {
//         // Iterate
//         for signed_sensor_data in signed_sensor_data_vec {
//             if let Some(sensor_data_hash) = verify_signature(&signed_sensor_data) {
//                 // Create SignatureContainer
//                 let sig_container = SignatureContainer {
//                     tceId: signed_sensor_data.tceId.clone(),
//                     sensorDataHash: sensor_data_hash.clone(),

//                 };

//                 sig_containers.push(sig_container);
//             } else {
//                 eprintln!("Signatur: UNGÜLTIG");
//                 continue;
//             }
//         }

//         // Serialize proof containers
//         let serialized_sig_containers = bincode
//             ::serialize(&sig_containers)
//             .expect("Failed to serialize proof_containers");

//         // Write to env_builder
//         env_builder
//             .write(&serialized_sig_containers)
//             .expect("Error while writing ProofContainers to Builder.");
//     }
// }
