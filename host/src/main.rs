use methods::{
    GUEST_BASELINE_ELF, GUEST_BASELINE_ID, GUEST_BASELINE_PRECOMPILES_ELF,
    GUEST_BASELINE_PRECOMPILES_ID, GUEST_BLS_PRECOMPILES_ELF, GUEST_BLS_PRECOMPILES_ID,
    GUEST_DUMMY_ELF, GUEST_DUMMY_ID, GUEST_LAZY_PRECOMPILES_ELF, GUEST_LAZY_PRECOMPILES_ID,
};

use base64::{engine::general_purpose, Engine as _};
use chrono::Local;
use env_helper::{
    get_configured_guest_mode, get_guest_type, process_and_write_proofs, ConfiguredGuestMode,
    GuestType,
};
use log::info;
use proving_service_core::product_footprint::ProductProof;
use proving_service_core::proofing_document::*;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use risc0_zkvm::sha::Digest;
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde_path_to_error::deserialize;
use std::fs::File;
use std::io::Write;
use tokio::time::Duration;
#[cfg(test)]
use tokio::time::Instant;

use crate::benchmarking::RunDataCollector;
use crate::bls_helper::aggregate_signatures;

mod benchmarking;
mod bls_helper;
mod env_helper;
mod sig_verifier;

/// Aggregates BLS signatures from all sensor data in the document.
fn aggregate_bls_signatures_from_document(document: &ProofingDocument) -> Vec<u8> {
    let signatures: Vec<Vec<u8>> = document
        .signedSensorData
        .as_ref()
        .map(|sensor_data_list| {
            sensor_data_list
                .iter()
                .filter(|sd| !sd.signedSensorData.is_empty())
                .filter_map(|sd| general_purpose::STANDARD.decode(&sd.signedSensorData).ok())
                .filter(|sig| sig.len() == 96) // BLS signature is 96 bytes
                .collect()
        })
        .unwrap_or_default();

    if signatures.is_empty() {
        return Vec::new();
    }

    aggregate_signatures(&signatures)
}

const TOPIC_IN: &str = "shipments";
const TOPIC_OUT: &str = "pcf-results";
const DEBUG: bool = false;

async fn process_payload(payload_str: &str) -> Option<ProductProof> {
    // println!("Rohdaten der Nachricht: {}", payload_str);
    // Versuch direkt zu parsen (raw JSON)
    if let Ok(proof_response) = try_handle_raw_json(payload_str).await {
        return Some(proof_response);
    }

    // Falls das fehlschlägt, versuche es als stringifizierten JSON-String zu entpacken
    let inner_json_str: String = match serde_json::from_str(payload_str) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Fehler beim Entpacken des JSON-Strings: {}", e);
            return None;
        }
    };

    try_handle_raw_json(&inner_json_str).await.ok()
}

async fn try_handle_raw_json(shipments_json: &str) -> Result<ProductProof, ()> {
    match handle_kafka_message(shipments_json).await {
        Some(resp) => Ok(resp),
        None => Err(()),
    }
}

#[tokio::main]
async fn main() {
    let brokers = std::env::var("KAFKA_BROKER").unwrap_or_else(|_| "localhost:9092".to_string());
    env_logger::init();

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("security.protocol", "PLAINTEXT")
        .set("group.id", "risc0-pcf-kafka-group")
        .set("auto.offset.reset", "earliest")
        .set("enable.auto.commit", "true")
        .set("auto.commit.interval.ms", "5000")
        .set("message.max.bytes", "52428800")
        .set("max.poll.interval.ms", "1800000")
        .create()
        .expect("Consumer creation failed");

    consumer.subscribe(&[TOPIC_IN]).unwrap();

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("security.protocol", "PLAINTEXT")
        .set("message.max.bytes", "52428800")
        .create()
        .expect("Producer creation failed");

    loop {
        match consumer.recv().await {
            Ok(message) => match message.payload_view::<str>() {
                Some(Ok(payload_str)) => {
                    if let Some(proof_response) = process_payload(payload_str).await {
                        let result_json = serde_json::to_string(&proof_response)
                            .expect("Failed to serialize proof_response");
                        let record = FutureRecord::to(TOPIC_OUT)
                            .payload(&result_json)
                            .key("some-key");
                        let _ = producer.send(record, Duration::from_secs(10)).await;
                    } else {
                        info!("Ungültige Nachricht wurde ignoriert.");
                    }
                }
                Some(Err(e)) => eprintln!("Payload UTF-8 error: {}", e),
                None => eprintln!("No payload"),
            },
            Err(e) => eprintln!("Kafka error receiving message: {:?}", e),
        }
    }
}

async fn main_proving_logic(
    mut proving_document: ProofingDocument,
    _collector: Option<&mut RunDataCollector>,
) -> Option<ProductProof> {
    #[cfg(test)] // Benchmarking
    let total_start_time = Instant::now();

    println!(
        "Received proving document with ID: {}",
        proving_document.productFootprint.id
    );
    println!(
        "From Company: {}",
        proving_document.productFootprint.companyName
    );

    // Take away the proof extension from the proving document
    let proof_vec = proving_document.proof;
    proving_document.proof = Vec::new();

    // Build the ExecutorEnv
    let mut builder = ExecutorEnv::builder();
    let executor_env_builder = builder
        .write(&proving_document)
        .expect("Failed to write proving_document to ExecutorEnv builder");

    // For BLS guest, aggregate signatures and write to env
    if get_configured_guest_mode() == ConfiguredGuestMode::BlsPrecompiles {
        let agg_sig = aggregate_bls_signatures_from_document(&proving_document);
        executor_env_builder
            .write(&agg_sig)
            .expect("Failed to write aggregated BLS signature");
    }

    process_and_write_proofs(&proof_vec, executor_env_builder);

    let env = executor_env_builder
        .build()
        .expect("Failed to build ExecutorEnv!");

    // Start the proving process
    let prover = default_prover();

    // Choose guest based on environment variable
    let (guest_elf, guest_id) = match get_configured_guest_mode() {
        ConfiguredGuestMode::Dummy => {
            println!("Using DUMMY guest");
            (GUEST_DUMMY_ELF, GUEST_DUMMY_ID)
        }
        ConfiguredGuestMode::BlsPrecompiles => {
            println!("Using BLS PRECOMPILES guest (aggregate signature verification)");
            (GUEST_BLS_PRECOMPILES_ELF, GUEST_BLS_PRECOMPILES_ID)
        }
        ConfiguredGuestMode::BaselinePrecompiles => {
            println!(
                "Using BASELINE PRECOMPILES guest (full signature verification in guest with precompiles)"
            );
            (
                GUEST_BASELINE_PRECOMPILES_ELF,
                GUEST_BASELINE_PRECOMPILES_ID,
            )
        }
        ConfiguredGuestMode::Baseline => {
            println!(
                "Using BASELINE guest (full signature verification in guest without precompiles)"
            );
            (GUEST_BASELINE_ELF, GUEST_BASELINE_ID)
        }
        ConfiguredGuestMode::LazyPrecompiles => {
            println!("Using LAZY guest (lazy signature verification)");
            (GUEST_LAZY_PRECOMPILES_ELF, GUEST_LAZY_PRECOMPILES_ID)
        }
    };

    println!("ELF size: {}", guest_elf.len());

    #[cfg(test)] // Benchmarking
    let proof_start_time = Instant::now();

    let prove_info = match prover.prove(env, guest_elf) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Error while proving: {}", e);
            return None;
        }
    };

    #[cfg(test)] // Benchmarking
    let duration = proof_start_time.elapsed();

    let receipt = prove_info.receipt;
    let guest_digest = Digest::from(guest_id);

    // Decode journal based on guest type
    let journal_output: f64 = match get_guest_type(&guest_digest) {
        GuestType::Dummy => {
            let verified_count: u64 = match receipt.journal.decode() {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Failed to decode journal: {}", e);
                    return None;
                }
            };
            println!("Dummy guest verified {} proof(s)", verified_count);
            verified_count as f64
        }
        GuestType::Baseline | GuestType::Bls => match receipt.journal.decode() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to decode journal: {}", e);
                return None;
            }
        },
        GuestType::Lazy => {
            let (pcf, _sigs): (f64, Vec<u8>) = match receipt.journal.decode() {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Failed to decode journal: {}", e);
                    return None;
                }
            };
            pcf
        }
        GuestType::Unknown => {
            eprintln!("Unknown guest type");
            return None;
        }
    };

    if let Err(e) = receipt.verify(guest_id) {
        eprintln!("Receipt verification failed: {}", e);
        return None;
    }

    let receipt_bytes = match bincode::serialize(&receipt) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to serialize receipt: {}", e);
            return None;
        }
    };
    let encoded_receipt = general_purpose::STANDARD.encode(receipt_bytes);

    println!("PCF Value from Journal: {}", journal_output);

    println!(
        "[{}]: Handed over response\n",
        Local::now().format("%H:%M:%S").to_string()
    );

    let proof_respone = ProductProof {
        productFootprintId: proving_document.productFootprint.id,
        proofReceipt: encoded_receipt,
        proofReference: "123".to_string(),
        pcf: journal_output,
        imageId: hex::encode(bytemuck::cast_slice(&guest_id)),
    };

    if DEBUG {
        // Write Output to file (for debugging purposes)
        let json_string = serde_json::to_string_pretty(&proof_respone).ok()?;
        let mut file = File::create("latest_output.json").ok()?;
        file.write_all(&json_string.as_bytes()).ok()?;
    }

    #[cfg(test)] // Benchmarking
    {
        let total_duration = total_start_time.elapsed();
        _collector
            .unwrap()
            .set_total_time(total_duration.as_secs())
            .set_proof_time(duration.as_secs())
            .set_cycles(&prove_info.stats);
    }

    Some(proof_respone)
}

async fn parse_proving_document(json_content: &str) -> Option<ProofingDocument> {
    let mut de = serde_json::Deserializer::from_str(json_content);
    match deserialize(&mut de) {
        Ok(proving_document) => Some(proving_document),
        Err(e) => {
            eprintln!(
                "Failed to deserialize message at path '{}': {}",
                e.path(),
                e
            );
            None
        }
    }
}

async fn handle_kafka_message(shipments_json: &str) -> Option<ProductProof> {
    println!(
        "[{}]: ----------- Received message -----------",
        Local::now().format("%H:%M:%S").to_string()
    );

    let proving_document = parse_proving_document(shipments_json)
        .await
        .expect("Failed to parse proving document");

    let product_proof = main_proving_logic(proving_document, None).await;

    product_proof
}

#[cfg(test)]
mod tests {
    use crate::{
        benchmarking::{create_numbered_file, DocumentGenerator, RunDataCollector},
        env_helper::{get_configured_guest_mode, ConfiguredGuestMode},
        main_proving_logic, parse_proving_document,
    };

    use super::handle_kafka_message;
    use proving_service_core::product_footprint::ProductProof;
    use rdkafka::{
        consumer::{Consumer as _, StreamConsumer},
        producer::{FutureProducer, FutureRecord},
        ClientConfig, Message as _,
    };
    use std::{
        env,
        fs::{self, File},
        io::Write,
        path::{Path, PathBuf},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    use tokio;

    const DEV_MODE: &str = "false";
    const SIGNATURE_COUNTS: [u32; 3] = [1, 4, 8];

    /// Find the most recent benchmark folder in benchmarks/documents/
    fn get_latest_benchmark_folder() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let base_dir = Path::new("../benchmarks/documents");

        let mut folders: Vec<PathBuf> = fs::read_dir(base_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_dir()
                    && path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.starts_with("benchmark_"))
                        .unwrap_or(false)
            })
            .collect();

        folders.sort();
        folders
            .last()
            .cloned()
            .ok_or_else(|| "No benchmark folder found. Run generate_benchmark_data first.".into())
    }

    fn get_latest_signature_test_folder() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let base_dir = Path::new("../benchmarks/documents");

        let mut folders: Vec<PathBuf> = fs::read_dir(base_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_dir()
                    && path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.starts_with("signature_test_"))
                        .unwrap_or(false)
            })
            .collect();

        folders.sort();
        folders.last().cloned().ok_or_else(|| {
            "No signature_test folder found. Run generate_signature_test_documents first.".into()
        })
    }

    #[tokio::test]
    async fn kafka_service() {
        const TOPIC_OUT: &str = "pcf-results";
        const TOPIC_IN: &str = "shipments";
        let brokers =
            std::env::var("KAFKA_BROKER").unwrap_or_else(|_| "localhost:9092".to_string());
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("security.protocol", "PLAINTEXT")
            .set("group.id", "risc0-pcf-kafka-group")
            .set("auto.offset.reset", "earliest")
            .set("enable.auto.commit", "true")
            .set("auto.commit.interval.ms", "5000")
            .set("message.max.bytes", "104857600")
            .create()
            .expect("Consumer creation failed");

        consumer.subscribe(&[TOPIC_OUT]).unwrap();

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("security.protocol", "PLAINTEXT")
            .set("message.max.bytes", "104857600")
            .create()
            .expect("Producer creation failed");
        let json_content = fs::read_to_string("../benchmarks/documents/comp_document_21.json");
        let binding = json_content.unwrap();
        let record = FutureRecord::to(TOPIC_OUT)
            .payload(&binding)
            .key("some-key");
        let _ = producer.send(record, Duration::from_secs(10)).await;

        loop {
            match consumer.recv().await {
                Ok(message) => match message.payload_view::<str>() {
                    Some(Ok(payload_str)) => {
                        println!("{}", payload_str);
                    }
                    Some(Err(e)) => eprintln!("Payload UTF-8 error: {}", e),
                    None => eprintln!("No payload"),
                },
                Err(e) => eprintln!("Kafka error receiving message: {:?}", e),
            }
        }
    }

    #[tokio::test]
    // Test: 3 TCEs; 1 Sig; 0 proofs
    async fn test_3_1_0() -> Result<(), Box<dyn std::error::Error>> {
        let json_content = fs::read_to_string("../benchmarks/documents/comp_document_5.json")?;

        // Call kafka handler
        let _resp: ProductProof = handle_kafka_message(&json_content)
            .await
            .expect("kafka_handler_failed");
        // If we reach here, resp is already a ProductProof, so no need to check is_some
        Ok(())
    }

    #[tokio::test]
    // Test: 3 TCEs; 1 Sig; 1 proofs
    async fn test_3_1_1() -> Result<(), Box<dyn std::error::Error>> {
        let json_content = fs::read_to_string("json-examples/test_3_1_1.json")?;

        // Call kafka handler
        let _resp: ProductProof = handle_kafka_message(&json_content)
            .await
            .expect("kafka_handler_failed");
        // If we reach here, resp is already a ProductProof, so no need to check is_some
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn generate_benchmark_data() -> Result<(), Box<dyn std::error::Error>> {
        use crate::benchmarking::{sign_document_bls, sign_document_rsa};

        let n: u32 = env::var("BENCHMARK_NUM_DOCS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15);
        let mut generator = DocumentGenerator::new();

        // Create timestamped folder
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let timestamp = chrono::DateTime::<chrono::Utc>::from(UNIX_EPOCH + now)
            .format("%Y%m%d_%H%M%S")
            .to_string();

        let benchmark_dir =
            Path::new("../benchmarks/documents").join(format!("benchmark_{}", timestamp));
        let base_docs_rsa_dir = benchmark_dir.join("base_documents_rsa");
        let base_docs_bls_dir = benchmark_dir.join("base_documents_bls");

        fs::create_dir_all(&base_docs_rsa_dir)?;
        fs::create_dir_all(&base_docs_bls_dir)?;

        println!(
            "Generating {} benchmark documents in {}...",
            n,
            benchmark_dir.display()
        );

        for i in 0..n {
            // Generate base document without signatures
            // let n_tocs = generator.gen_range(5..9);
            // let m_hocs = generator.gen_range(2..5);
            let n_tocs = 4;
            let m_hocs = 2;
            let base_document = generator.generate_base_document(n_tocs, m_hocs);

            // Sign with RSA and save
            let rsa_document = sign_document_rsa(base_document.clone());
            let rsa_path = base_docs_rsa_dir.join(format!("base_document_{}.json", i));
            let mut rsa_file = File::create(&rsa_path)?;
            let rsa_json = serde_json::to_string_pretty(&rsa_document)?;
            rsa_file.write_all(rsa_json.as_bytes())?;

            // Sign with BLS and save
            let bls_document = sign_document_bls(base_document);
            let bls_path = base_docs_bls_dir.join(format!("base_document_{}.json", i));
            let mut bls_file = File::create(&bls_path)?;
            let bls_json = serde_json::to_string_pretty(&bls_document)?;
            bls_file.write_all(bls_json.as_bytes())?;

            println!("Created document {}: RSA and BLS", i);
        }

        println!(
            "Successfully generated {} benchmark documents (RSA + BLS) in {}",
            n,
            benchmark_dir.display()
        );
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn generate_signature_test_documents() -> Result<(), Box<dyn std::error::Error>> {
        use crate::benchmarking::{sign_document_bls, sign_document_rsa};

        let mut generator = DocumentGenerator::new();

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let timestamp = chrono::DateTime::<chrono::Utc>::from(UNIX_EPOCH + now)
            .format("%Y%m%d_%H%M%S")
            .to_string();

        let sig_test_dir =
            Path::new("../benchmarks/documents").join(format!("signature_test_{}", timestamp));
        let base_docs_rsa_dir = sig_test_dir.join("base_documents_rsa");
        let base_docs_bls_dir = sig_test_dir.join("base_documents_bls");
        fs::create_dir_all(&base_docs_rsa_dir)?;
        fs::create_dir_all(&base_docs_bls_dir)?;

        for sig_count in SIGNATURE_COUNTS {
            // Generate base document without signatures
            let base_document = generator.generate_base_document(sig_count, 0);

            // Sign with RSA and save
            let rsa_document = sign_document_rsa(base_document.clone());
            let rsa_path = base_docs_rsa_dir.join(format!("sig_document_{}.json", sig_count));
            let mut rsa_file = File::create(&rsa_path)?;
            let rsa_json = serde_json::to_string_pretty(&rsa_document)?;
            rsa_file.write_all(rsa_json.as_bytes())?;

            // Sign with BLS and save
            let bls_document = sign_document_bls(base_document);
            let bls_path = base_docs_bls_dir.join(format!("sig_document_{}.json", sig_count));
            let mut bls_file = File::create(&bls_path)?;
            let bls_json = serde_json::to_string_pretty(&bls_document)?;
            bls_file.write_all(bls_json.as_bytes())?;

            println!(
                "Created document with {} signature(s): RSA and BLS",
                sig_count
            );
        }

        println!("Documents created in: {}", sig_test_dir.display());
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_signatures() -> Result<(), Box<dyn std::error::Error>> {
        env::set_var("RISC0_DEV_MODE", DEV_MODE);

        let dir_name = match get_configured_guest_mode() {
            ConfiguredGuestMode::BlsPrecompiles => "signatures_bls_precompiles",
            ConfiguredGuestMode::BaselinePrecompiles => "signatures_baseline_precompiles",
            ConfiguredGuestMode::Baseline => "signatures_baseline",
            ConfiguredGuestMode::Dummy => "signatures_dummy",
            ConfiguredGuestMode::LazyPrecompiles => "signatures_lazy_precompiles",
        };

        let mut collector = RunDataCollector::new(dir_name);
        let mut verify_collector = RunDataCollector::new(format!("{}_verify", dir_name));
        let sig_test_dir = get_latest_signature_test_folder()?;

        // Use BLS documents for BLS guest, RSA documents for others
        let base_docs_dir = match get_configured_guest_mode() {
            ConfiguredGuestMode::BlsPrecompiles => sig_test_dir.join("base_documents_bls"),
            _ => sig_test_dir.join("base_documents_rsa"),
        };

        let proofs_dir = sig_test_dir.join(dir_name);
        fs::create_dir_all(&proofs_dir)?;

        // Generator for empty carrier documents
        let mut generator = DocumentGenerator::new();

        println!(
            "Running signature tests ({}) from {}...",
            dir_name,
            sig_test_dir.display()
        );

        for sig_count in SIGNATURE_COUNTS {
            let path = base_docs_dir.join(format!("sig_document_{}.json", sig_count));
            let json_content = fs::read_to_string(&path)?;
            let proving_document = parse_proving_document(&json_content)
                .await
                .expect("Failed to parse proving document");

            println!("\nSignature count: {}", sig_count);
            println!("Generating proof");

            env::set_var("USE_DUMMY_GUEST", "false");
            collector.start_new_run().set_input(&proving_document);
            let resp = main_proving_logic(proving_document.clone(), Some(&mut collector))
                .await
                .expect("Failed main logic");
            collector.set_output(&resp);
            collector.print_current_run();

            // Save the proof
            let proof_path = proofs_dir.join(format!("sig_proof_{}.json", sig_count));
            let mut file = File::create(&proof_path)?;
            let json_string = serde_json::to_string_pretty(&resp)?;
            file.write_all(json_string.as_bytes())?;
            println!("  Saved proof to: {}", proof_path.display());

            collector
                .write_to_csv_with_path(&sig_test_dir)
                .expect("Failed to write metrics to CSV");

            // Send proof to dummy guest for verification
            println!("Verifying proof with dummy guest...");

            env::set_var("USE_DUMMY_GUEST", "true");
            let mut dummy_document = generator.generate_base_document(0, 0);
            dummy_document.proof.push(resp);

            verify_collector
                .start_new_run()
                .set_input(&proving_document); // Use original doc for signature count
            let verify_resp =
                main_proving_logic(dummy_document.clone(), Some(&mut verify_collector))
                    .await
                    .expect("Verify proving failed");
            verify_collector.set_output(&verify_resp);
            verify_collector.print_current_run();

            // Save dummy result
            let verify_proof_path = proofs_dir.join(format!("verify_proof_{}.json", sig_count));
            let mut file = File::create(&verify_proof_path)?;
            let json_string = serde_json::to_string_pretty(&verify_resp)?;
            file.write_all(json_string.as_bytes())?;
            println!("  Saved verify proof to: {}", verify_proof_path.display());

            verify_collector
                .write_to_csv_with_path(&sig_test_dir)
                .expect("Failed to write verify metrics to CSV");
        }

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn bench_composition() -> Result<(), Box<dyn std::error::Error>> {
        env::set_var("RISC0_DEV_MODE", DEV_MODE);
        let n: u32 = env::var("BENCHMARK_NUM_DOCS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15);

        let (test_name, dir_name) = match get_configured_guest_mode() {
            ConfiguredGuestMode::BlsPrecompiles => (
                "bench_composition_bls_precompiles",
                "composition_bls_precompiles",
            ),
            ConfiguredGuestMode::BaselinePrecompiles => (
                "bench_composition_baseline_precompiles",
                "composition_baseline_precompiles",
            ),
            ConfiguredGuestMode::Baseline => ("bench_composition_baseline", "composition_baseline"),
            ConfiguredGuestMode::Dummy => ("bench_composition_dummy", "composition_dummy"),
            ConfiguredGuestMode::LazyPrecompiles => (
                "bench_composition_lazy_precompiles",
                "composition_lazy_precompiles",
            ),
        };
        let mut collector = RunDataCollector::new(test_name);
        let mut response: Option<ProductProof> = None;

        // Get the latest benchmark folder
        let benchmark_dir = get_latest_benchmark_folder()?;

        // Use BLS documents for BLS guest, RSA documents for others
        let base_docs_dir = match get_configured_guest_mode() {
            ConfiguredGuestMode::BlsPrecompiles => benchmark_dir.join("base_documents_bls"),
            _ => benchmark_dir.join("base_documents_rsa"),
        };

        let composition_dir = benchmark_dir.join(dir_name);
        fs::create_dir_all(&composition_dir)?;

        println!(
            "Running composition benchmark with {} documents from {}...",
            n,
            benchmark_dir.display()
        );

        for i in 0..n {
            let path = base_docs_dir.join(format!("base_document_{}.json", i));
            let json_content = fs::read_to_string(&path)?;
            let mut proving_document = parse_proving_document(&json_content)
                .await
                .expect("Failed to parse proving document");

            // Add previous proof for composition
            if let Some(ref resp) = response {
                proving_document.proof.push(resp.clone());
            }

            collector.start_new_run().set_input(&proving_document);
            response = Some(
                main_proving_logic(proving_document.clone(), Some(&mut collector))
                    .await
                    .expect("Failed main logic"),
            );
            collector.set_output(response.as_ref().unwrap());
            collector.print_current_run();

            // Save the proof to the composition folder
            if let Some(ref resp) = response {
                let proof_path = composition_dir.join(format!("comp_proof_{}.json", i));
                let mut file = File::create(&proof_path)?;
                let json_string = serde_json::to_string_pretty(resp)?;
                file.write_all(json_string.as_bytes())?;
            }
            collector
                .write_to_csv_with_path(&benchmark_dir)
                .expect("Failed to write metrics to CSV");
        }
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn bench_aggregation() -> Result<(), Box<dyn std::error::Error>> {
        env::set_var("RISC0_DEV_MODE", DEV_MODE);
        let n: u32 = env::var("BENCHMARK_NUM_DOCS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15);

        let mut generator = DocumentGenerator::new();

        let (test_name, dir_name) = match get_configured_guest_mode() {
            ConfiguredGuestMode::BlsPrecompiles => (
                "bench_aggregation_bls_precompiles",
                "aggregation_bls_precompiles",
            ),
            ConfiguredGuestMode::BaselinePrecompiles => (
                "bench_aggregation_baseline_precompiles",
                "aggregation_baseline_precompiles",
            ),
            ConfiguredGuestMode::Baseline => ("bench_aggregation_baseline", "aggregation_baseline"),
            ConfiguredGuestMode::Dummy => ("bench_aggregation_dummy", "aggregation_dummy"),
            ConfiguredGuestMode::LazyPrecompiles => (
                "bench_aggregation_lazy_precompiles",
                "aggregation_lazy_precompiles",
            ),
        };
        let mut collector = RunDataCollector::new(test_name);
        let mut blank_proving_document = generator.generate_base_document(0, 0);

        // Get the latest benchmark folder
        let benchmark_dir = get_latest_benchmark_folder()?;

        // Use BLS documents for BLS guest, RSA documents for others
        let base_docs_dir = match get_configured_guest_mode() {
            ConfiguredGuestMode::BlsPrecompiles => benchmark_dir.join("base_documents_bls"),
            _ => benchmark_dir.join("base_documents_rsa"),
        };

        let aggregation_dir = benchmark_dir.join(dir_name);
        fs::create_dir_all(&aggregation_dir)?;

        println!(
            "Running aggregation benchmark with {} documents from {}...",
            n,
            benchmark_dir.display()
        );

        for i in 0..n {
            let path = base_docs_dir.join(format!("base_document_{}.json", i));
            let json_content = fs::read_to_string(path)?;
            let mut proving_document = parse_proving_document(&json_content)
                .await
                .expect("Failed to parse proving document");

            // Aggregate all TCE data from each document
            blank_proving_document
                .tocData
                .append(&mut proving_document.tocData);
            blank_proving_document
                .hocData
                .append(&mut proving_document.hocData);
            match (
                &mut blank_proving_document.signedSensorData,
                &mut proving_document.signedSensorData,
            ) {
                (Some(blank_vec), Some(proving_vec)) => blank_vec.append(proving_vec),
                (None, Some(proving_vec)) if !proving_vec.is_empty() => {
                    blank_proving_document.signedSensorData = Some(std::mem::take(proving_vec));
                }
                _ => {}
            }
            blank_proving_document.productFootprint.extensions[0]
                .data
                .tces
                .append(&mut proving_document.productFootprint.extensions[0].data.tces);
        }

        // Save the aggregated document
        let aggregated_doc_path = aggregation_dir.join("aggregated_document.json");
        let mut file = File::create(&aggregated_doc_path)?;
        let json_string = serde_json::to_string_pretty(&blank_proving_document)?;
        file.write_all(json_string.as_bytes())?;

        collector.start_new_run().set_input(&blank_proving_document);
        let response =
            main_proving_logic(blank_proving_document.clone(), Some(&mut collector)).await;
        collector.set_output(response.as_ref().unwrap());
        collector.print_current_run();

        // Save the aggregation proof
        if let Some(ref resp) = response {
            let proof_path = aggregation_dir.join("aggregation_proof.json");
            let mut file = File::create(&proof_path)?;
            let json_string = serde_json::to_string_pretty(resp)?;
            file.write_all(json_string.as_bytes())?;
        }

        collector
            .write_to_csv_with_path(&benchmark_dir)
            .expect("Failed to write metrics to CSV");
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn bench_proofaggregation() -> Result<(), Box<dyn std::error::Error>> {
        env::set_var("RISC0_DEV_MODE", DEV_MODE);
        let n: u32 = env::var("BENCHMARK_NUM_DOCS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15);

        let (test_name, dir_name) = match get_configured_guest_mode() {
            ConfiguredGuestMode::BlsPrecompiles => (
                "bench_proofaggregation_bls_precompiles",
                "proof_aggregation_bls_precompiles",
            ),
            ConfiguredGuestMode::BaselinePrecompiles => (
                "bench_proofaggregation_baseline_precompiles",
                "proof_aggregation_baseline_precompiles",
            ),
            ConfiguredGuestMode::Baseline => (
                "bench_proofaggregation_baseline",
                "proof_aggregation_baseline",
            ),
            ConfiguredGuestMode::Dummy => {
                ("bench_proofaggregation_dummy", "proof_aggregation_dummy")
            }
            ConfiguredGuestMode::LazyPrecompiles => (
                "bench_proofaggregation_lazy_precompiles",
                "proof_aggregation_lazy_precompiles",
            ),
        };
        let mut collector = RunDataCollector::new(test_name);
        let mut previous_proofs: Vec<ProductProof> = Vec::new();

        // Get the latest benchmark folder
        let benchmark_dir = get_latest_benchmark_folder()?;

        // Use BLS documents for BLS guest, RSA documents for others
        let base_docs_dir = match get_configured_guest_mode() {
            ConfiguredGuestMode::BlsPrecompiles => benchmark_dir.join("base_documents_bls"),
            _ => benchmark_dir.join("base_documents_rsa"),
        };

        let proof_aggr_dir = benchmark_dir.join(dir_name);
        fs::create_dir_all(&proof_aggr_dir)?;

        println!(
            "Running proof aggregation benchmark with {} documents from {}...",
            n,
            benchmark_dir.display()
        );

        // Generate individual proofs for documents 0 to n-2
        for i in 0..(n - 1) {
            let path = base_docs_dir.join(format!("base_document_{}.json", i));
            let json_content = fs::read_to_string(path)?;
            let proving_document = parse_proving_document(&json_content)
                .await
                .expect("Failed to parse proving document");

            collector.start_new_run().set_input(&proving_document);
            let response = main_proving_logic(proving_document.clone(), Some(&mut collector)).await;
            collector.set_output(response.as_ref().unwrap());
            collector.print_current_run();

            // Save individual proof
            if let Some(ref resp) = response {
                let proof_path = proof_aggr_dir.join(format!("individual_proof_{}.json", i));
                let mut file = File::create(&proof_path)?;
                let json_string = serde_json::to_string_pretty(resp)?;
                file.write_all(json_string.as_bytes())?;
            }

            collector
                .write_to_csv_with_path(&benchmark_dir)
                .expect("Failed to write metrics to CSV");

            previous_proofs.push(response.unwrap().clone());
        }

        // Final proof (document n-1): verifies all previous proofs AND processes its own data
        let final_doc_index = n - 1;
        let path = base_docs_dir.join(format!("base_document_{}.json", final_doc_index));
        let json_content = fs::read_to_string(path)?;
        let mut final_proving_document = parse_proving_document(&json_content)
            .await
            .expect("Failed to parse proving document");

        // Add all previous proofs to the final document
        final_proving_document.proof = previous_proofs;

        // Save the aggregated proof document (contains all previous proofs + final document data)
        let aggr_doc_path = proof_aggr_dir.join("proof_aggr_document.json");
        let mut file = File::create(&aggr_doc_path)?;
        let json_string = serde_json::to_string_pretty(&final_proving_document)?;
        file.write_all(json_string.as_bytes())?;

        // Generate the final proof that verifies all previous proofs and processes document 19
        collector.start_new_run().set_input(&final_proving_document);
        let response =
            main_proving_logic(final_proving_document.clone(), Some(&mut collector)).await;
        collector.set_output(response.as_ref().unwrap());
        collector.print_current_run();

        // Save the final aggregated proof
        if let Some(ref resp) = response {
            // Build the list of child document indices
            let child_indices: Vec<String> = (0..(n - 1)).map(|i| i.to_string()).collect();
            let children_list = child_indices.join("_");
            let final_proof_path = proof_aggr_dir.join(format!(
                "parent_proof_from_documents_{}.json",
                children_list
            ));
            let mut file = File::create(&final_proof_path)?;
            let json_string = serde_json::to_string_pretty(resp)?;
            file.write_all(json_string.as_bytes())?;
        }

        collector
            .write_to_csv_with_path(&benchmark_dir)
            .expect("Failed to write metrics to CSV");
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn bench_tree_aggregation() -> Result<(), Box<dyn std::error::Error>> {
        env::set_var("RISC0_DEV_MODE", DEV_MODE);

        let total_documents: u32 = env::var("BENCHMARK_NUM_DOCS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15);

        // Calculate number of leaves
        let num_leaves: u32 = (total_documents + 1) / 2;

        let (test_name, dir_name) = match get_configured_guest_mode() {
            ConfiguredGuestMode::BlsPrecompiles => (
                "bench_tree_aggregation_bls_precompiles",
                "tree_aggregation_bls_precompiles",
            ),
            ConfiguredGuestMode::BaselinePrecompiles => (
                "bench_tree_aggregation_baseline_precompiles",
                "tree_aggregation_baseline_precompiles",
            ),
            ConfiguredGuestMode::Baseline => (
                "bench_tree_aggregation_baseline",
                "tree_aggregation_baseline",
            ),
            ConfiguredGuestMode::Dummy => {
                ("bench_tree_aggregation_dummy", "tree_aggregation_dummy")
            }
            ConfiguredGuestMode::LazyPrecompiles => (
                "bench_tree_aggregation_lazy_precompiles",
                "tree_aggregation_lazy_precompiles",
            ),
        };
        let mut collector = RunDataCollector::new(test_name);

        // Get the latest benchmark folder
        let benchmark_dir = get_latest_benchmark_folder()?;

        // Use BLS documents for BLS guest, RSA documents for others
        let base_docs_dir = match get_configured_guest_mode() {
            ConfiguredGuestMode::BlsPrecompiles => benchmark_dir.join("base_documents_bls"),
            _ => benchmark_dir.join("base_documents_rsa"),
        };

        let tree_aggr_dir = benchmark_dir.join(dir_name);
        fs::create_dir_all(&tree_aggr_dir)?;

        // Map to store proofs by document index
        let mut proofs: std::collections::HashMap<u32, ProductProof> =
            std::collections::HashMap::new();

        // Level 0: Generate individual proofs for leaf documents
        println!(
            "\nLevel 0: Generating {} leaf proofs (docs 0-{})",
            num_leaves,
            num_leaves - 1
        );
        for i in 0..num_leaves {
            let path = base_docs_dir.join(format!("base_document_{}.json", i));
            let json_content = fs::read_to_string(path)?;
            let proving_document = parse_proving_document(&json_content)
                .await
                .expect("Failed to parse proving document");

            collector.start_new_run().set_input(&proving_document);
            let response = main_proving_logic(proving_document.clone(), Some(&mut collector))
                .await
                .expect("Failed to generate leaf proof");
            collector.set_output(&response);
            collector.print_current_run();

            // Save leaf proof
            let proof_path = tree_aggr_dir.join(format!("level_0_doc_{}.json", i));
            let mut file = File::create(&proof_path)?;
            let json_string = serde_json::to_string_pretty(&response)?;
            file.write_all(json_string.as_bytes())?;

            collector
                .write_to_csv_with_path(&benchmark_dir)
                .expect("Failed to write metrics to CSV");

            proofs.insert(i, response);
        }

        // Build parent levels dynamically
        let mut current_level = 0;
        let mut level_start = 0_u32;
        let mut level_count = num_leaves;
        let mut next_doc_index = num_leaves;

        while level_count > 1 {
            current_level += 1;
            let next_level_count = level_count / 2;

            println!(
                "\nLevel {}: Generating {} parent proofs (docs {}-{})",
                current_level,
                next_level_count,
                next_doc_index,
                next_doc_index + next_level_count - 1
            );

            for i in 0..next_level_count {
                let doc_index = next_doc_index + i;
                let left_child = level_start + (i * 2);
                let right_child = level_start + (i * 2) + 1;

                let path = base_docs_dir.join(format!("base_document_{}.json", doc_index));
                let json_content = fs::read_to_string(path)?;
                let mut proving_document = parse_proving_document(&json_content)
                    .await
                    .expect("Failed to parse proving document");

                // Add child proofs
                proving_document
                    .proof
                    .push(proofs.get(&left_child).unwrap().clone());
                proving_document
                    .proof
                    .push(proofs.get(&right_child).unwrap().clone());

                collector.start_new_run().set_input(&proving_document);
                let response = main_proving_logic(proving_document.clone(), Some(&mut collector))
                    .await
                    .expect("Failed to generate proof");
                collector.set_output(&response);
                collector.print_current_run();

                let proof_path = tree_aggr_dir.join(format!(
                    "level_{}_doc_{}_from_{}_and_{}.json",
                    current_level, doc_index, left_child, right_child
                ));
                let mut file = File::create(&proof_path)?;
                let json_string = serde_json::to_string_pretty(&response)?;
                file.write_all(json_string.as_bytes())?;

                collector
                    .write_to_csv_with_path(&benchmark_dir)
                    .expect("Failed to write metrics to CSV");

                proofs.insert(doc_index, response);
            }

            level_start += level_count;
            level_count = next_level_count;
            next_doc_index += next_level_count;
        }

        println!("\nTree aggregation complete");
        println!(
            "Perfect binary tree: {} documents, {} levels",
            total_documents,
            current_level + 1
        );

        Ok(())
    }
}
