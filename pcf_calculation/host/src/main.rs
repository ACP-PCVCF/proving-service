use methods::{GUEST_CODE_FOR_ZK_PROOF_ELF, GUEST_CODE_FOR_ZK_PROOF_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty}; 
use std::fs;

mod verify; 

#[derive(Deserialize, Serialize)]
struct Activity {
    process_id: String,
    unit: String,
    consumption: u32,
    e_type: String,
}

#[derive(Deserialize, Serialize)]
struct ShipmentInfo {
    activity_data_json: String,
    activity_signature: String,
    activity_public_key_pem: String,
}

#[derive(Deserialize, Serialize)]
struct Shipment {
    shipment_id: String,
    info: ShipmentInfo,
}

#[derive(Deserialize, Serialize)]
struct CombinedInput {
    activities: Vec<Activity>,
    shipments: Vec<Shipment>,
}

#[derive(Serialize)]
struct ReceiptExport {
    image_id: String,
    receipt: risc0_zkvm::Receipt,
}

fn main() {
    
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let activity_json =
        fs::read_to_string("host/src/activity.json").expect("File was not readable!!!");

    let activities: Vec<Activity> = from_str(&activity_json).unwrap();

    let shipment_json =
        fs::read_to_string("host/src/shipments.json").expect("Shipments file not readable");

    let shipments: Vec<Shipment> = from_str(&shipment_json).unwrap();

    let combined_input = CombinedInput {
        activities,
        shipments,
    };

    let env = ExecutorEnv::builder()
        .write(&combined_input)
        .expect("Failed to write combined input to ExecutorEnv")
        .build()
        .unwrap();

    let prover = default_prover();

    let prove_info = prover.prove(env, GUEST_CODE_FOR_ZK_PROOF_ELF).unwrap();

    let receipt = prove_info.receipt;

    let pcf_total: u32 = receipt.journal.decode().unwrap();

    receipt.verify(GUEST_CODE_FOR_ZK_PROOF_ID).unwrap();

    print!(
        "The total CO2-Emission for the process pID-3423452 is {} kg CO2e",
        { pcf_total }
    );

    let image_id_digest = Digest::from(GUEST_CODE_FOR_ZK_PROOF_ID);
    let image_id_hex = image_id_digest.to_string();

    let export = ReceiptExport {
        image_id: image_id_hex,
        receipt,
    };

    let receipt_json = to_string_pretty(&export).expect("JSON serialization failed");

    fs::write("receipt_output.json", receipt_json).expect("Couldn't write receipt_output.json");

    println!("Receipt + Image ID gespeichert in: receipt_output.json");

    if let Err(e) = verify::verify_receipt() {
        eprintln!("❌ Fehler bei der Verifikation: {:?}", e);
    }
}
