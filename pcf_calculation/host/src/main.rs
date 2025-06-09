use methods::{GUEST_CODE_FOR_ZK_PROOF_ELF, GUEST_CODE_FOR_ZK_PROOF_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, Digest};
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

//#[derive(Deserialize, Serialize)]
//struct ShipmentInfo {
//    activity_data_json: String,
//    activity_signature: String,
//    activity_public_key_pem: String,
//}

#[derive(Deserialize, Debug)]
struct OgJsonTopLevel {
    #[serde(rename = "productFootprint")]
    product_footprint: serde_json::Value, // Oder eine detailliertere Struktur
    #[serde(rename = "tocData")]
    toc_data: Vec<serde_json::Value>, // Oder eine detailliertere Struktur
    #[serde(rename = "hocData")]
    hoc_data: Vec<serde_json::Value>, // Oder eine detailliertere Struktur
    #[serde(rename = "signedSensorData")]
    signed_sensor_data_list: Vec<SignedSensorData>, // Hier ist Ihre Liste
}

// Diese Strukturen müssen sowohl im Host als auch im Gast-Code
// (falls SignedSensorData dort verwendet wird und dieselbe Struktur hat)
// definiert oder importiert werden.

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Distance {
    actual: f64, // oder f32, je nach Genauigkeit. JSON 'number' wird oft zu f64.
    gcd: Option<f64>, // Da es 'null' sein kann
    sfd: Option<f64>, // Da es 'null' sein kann
}

/*#[derive(Deserialize, Serialize, Debug, Clone)]
struct SensorDataPayload {
    distance: Distance,
}*/

// Ihre SignedSensorData-Struktur wird dann angepasst:
#[derive(Deserialize, Serialize, Debug, Clone)]
struct SignedSensorData {
    #[serde(rename = "tceId")]
    tce_id: String,
    #[serde(rename = "camundaProcessInstanceKey")]
    camunda_process_instance_key: String,
    #[serde(rename = "camundaActivityId")]
    camunda_activity_id: String,
    sensorkey: String,
    #[serde(rename = "signedSensorData")]
    signed_sensor_data: String, // Behält den ursprünglichen Namen für Klarheit
    #[serde(rename = "sensorData")]
    //sensor_data: SensorDataPayload, // GEÄNDERT: von String zu SensorDataPayload
    sensor_data: String
}

#[derive(Deserialize, Serialize)]
struct CombinedInput {
    activities: Vec<Activity>,
    //shipments: Vec<Shipment>,
    signatures: Vec<SignedSensorData>,
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

    let signatures_json =
        fs::read_to_string("host/src/og.json").expect("Sensor file not readable");
    // Zuerst in ein Array von OgJsonTopLevel-Objekten deserialisieren
    let top_level_data_vec: Vec<OgJsonTopLevel> = from_str(&signatures_json).unwrap();

    // Wenn Sie sicher sind, dass es nur ein Element im äußeren Array von og.json gibt:
    let signatures: Vec<SignedSensorData> = if let Some(top_level_data) = top_level_data_vec.get(0) {
        top_level_data.signed_sensor_data_list.clone()
    } else {
        panic!("og.json ist leer oder hat nicht das erwartete Format");
    };

    let combined_input = CombinedInput {
        activities,
        signatures,
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
