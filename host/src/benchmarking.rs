#![allow(dead_code)]

use crate::sig_verifier::hash;
use base64::engine::general_purpose;
use base64::Engine as _;
use pkcs1::{EncodeRsaPublicKey as _, LineEnding};
use proving_service_core::hoc_toc_data::{HocData, TocData, TransportMode};
use proving_service_core::product_footprint::{self, Distance, ProductFootprint, TCE};
use proving_service_core::proofing_document::{SensorData, TceSensorData};
use proving_service_core::{product_footprint::ProductProof, proofing_document::ProofingDocument};
use rand::rngs::{OsRng, ThreadRng};
use rand::{Rng as _, RngCore as _};
use risc0_zkvm::SessionStats;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Serialize)]
struct RunMetrics {
    run_id: u64,
    time: u64,
    input_size: u64,
    output_size: u64,
    paging_cycles: u64,
    user_cycles: u64,
    reserved_cycles: u64,
    total_cycles: u64,
}

pub struct RunDataCollector {
    test_name: String,
    data: Vec<RunMetrics>,
}

impl RunDataCollector {
    pub fn new(test_name: impl Into<String>) -> Self {
        RunDataCollector {
            test_name: test_name.into(),
            data: Vec::new(),
        }
    }

    pub fn start_new_run(&mut self) -> &mut RunDataCollector {
        let run_id = self.data.len() as u64 + 1;
        self.data.push(RunMetrics {
            run_id,
            time: 0,
            input_size: 0,
            output_size: 0,
            paging_cycles: 0,
            user_cycles: 0,
            reserved_cycles: 0,
            total_cycles: 0,
        });
        self
    }

    pub fn set_time(&mut self, elapsed: u64) -> &mut RunDataCollector {
        self.data.last_mut().map(|metrics| metrics.time = elapsed);
        self
    }

    pub fn set_input(&mut self, document: &ProofingDocument) -> &mut RunDataCollector {
        let size: u64 = serde_json::to_string(document)
            .map(|s| s.len() as u64)
            .unwrap_or(0);
        self.data
            .last_mut()
            .map(|metrics| metrics.input_size = size);
        self
    }

    pub fn set_output(&mut self, response: &ProductProof) -> &mut RunDataCollector {
        let size: u64 = serde_json::to_string(response)
            .map(|s| s.len() as u64)
            .unwrap_or(0);
        self.data
            .last_mut()
            .map(|metrics| metrics.output_size = size);
        self
    }

    pub fn set_cycles(&mut self, stats: &SessionStats) -> &mut RunDataCollector {
        if let Some(metrics) = self.data.last_mut() {
            metrics.paging_cycles = stats.paging_cycles;
            metrics.user_cycles = stats.user_cycles;
            metrics.reserved_cycles = stats.reserved_cycles;
            metrics.total_cycles = stats.total_cycles;
        }
        self
    }

    pub fn write_to_csv(&self) -> Result<(), Box<dyn Error>> {
        let output_dir = Path::new("benchmarks");
        let base_file_name = Path::new(&self.test_name);
        let extension = "csv";

        fs::create_dir_all(output_dir)?;
        let full_base_path = output_dir.join(base_file_name);

        match create_numbered_file(&full_base_path, extension) {
            Ok(path) => {
                let file = File::create(&path)?;
                let mut wtr = csv::Writer::from_writer(file);

                wtr.write_record(&[
                    "run_id",
                    "time",
                    "input_size",
                    "output_size",
                    "paging_cycles",
                    "user_cycles",
                    "reserved_cycles",
                    "total_cycles",
                ])?;

                for metrics in &self.data {
                    wtr.serialize((
                        metrics.run_id,
                        metrics.time,
                        metrics.input_size,
                        metrics.output_size,
                        metrics.paging_cycles,
                        metrics.user_cycles,
                        metrics.reserved_cycles,
                        metrics.total_cycles,
                    ))?;
                }

                wtr.flush()?;
            }
            Err(_) => { /* Do nothing, just return unit */ }
        }

        Ok(())
    }
}

pub struct DocumentGenerator {
    rng: ThreadRng,
}

impl DocumentGenerator {
    pub fn new() -> Self {
        DocumentGenerator {
            rng: rand::thread_rng(),
        }
    }

    fn generate_random_toc(&mut self) -> TocData {
        TocData {
            tocId: self.rng.gen_range(0..10000).to_string(),
            certifications: Vec::new(),
            description: "None".to_string(),
            mode: TransportMode::Road,
            loadFactor: 1.to_string(),
            emptyDistanceFactor: 1.to_string(),
            temperatureControl: "None".to_string(),
            truckLoadingSequence: "None".to_string(),
            airShippingOption: None,
            flightLength: None,
            energyCarriers: Vec::new(),
            co2eIntensityWTW: self.rng.gen_range(0..100).to_string(),
            co2eIntensityTTW: "None".to_string(),
            transportActivityUnit: "None".to_string(),
        }
    }

    fn generate_random_hoc(&mut self) -> HocData {
        HocData {
            hocId: self.rng.gen_range(0..10000).to_string(),
            passhubType: "None".to_string(),
            energyCarriers: Vec::new(),
            co2eIntensityWTW: self.rng.gen_range(0..100).to_string(),
            co2eIntensityTTW: "None".to_string(),
            hubActivityUnit: "None".to_string(),
        }
    }

    pub fn generate_proving_document_random(&mut self) -> ProofingDocument {
        let n = self.rng.gen_range(1..4);
        let m = std::cmp::max(0, n - self.rng.gen_range(0..2));
        self.generate_proving_document(m, n)
    }

    pub fn generate_proving_document(&mut self, n: u32, m: u32) -> ProofingDocument {
        let mut rng = OsRng;
        let bits = 2048;

        let shipment_id = self.rng.gen_range(0..10000).to_string();
        let mass: f64 = self.rng.gen_range(10.0..1000.0);
        let mut tces: Vec<TCE> = Vec::new();
        let mut tocs: Vec<TocData> = Vec::new();
        let mut hocs: Vec<HocData> = Vec::new();
        let mut ssd: Vec<TceSensorData> = Vec::new();

        for _ in 0..n {
            let distance = Distance {
                actual: self.rng.gen_range(1.0..1000.0),
                gcd: None,
                sfd: None,
            };

            let toc = self.generate_random_toc();
            let tce: TCE = TCE {
                tceId: self.rng.gen_range(0..10000).to_string(),
                prevTceIds: tces.iter().map(|t| t.tceId.clone()).collect(),
                hocId: None,
                tocId: Some(toc.tocId.clone()),
                shipmentId: shipment_id.clone(),
                mass: mass.clone(),
                co2eWTW: None,
                co2eTTW: None,
                transportActivity: None,
                distance: Some(distance.clone()),
            };

            let sensor_data: SensorData = SensorData {
                distance: distance.clone(),
            };

            let private_key = RsaPrivateKey::new(&mut rng, bits)
                .expect("Fehler beim Erstellen des RSA Privatschlüssels");
            let public_key = RsaPublicKey::from(&private_key);

            let mut salt = vec![0u8; 32];
            rng.fill_bytes(&mut salt);
            let salt_base64 = general_purpose::STANDARD.encode(&salt);

            let data = format!(
                "{}{}",
                serde_json::to_string(&sensor_data).unwrap(),
                salt_base64
            );
            let commitment = hash(&data);

            let encrypted_data = public_key
                .encrypt(&mut rng, Pkcs1v15Encrypt, &commitment)
                .expect("Fehler beim Verschlüsseln der Nachricht");

            let encrypted_data_base64 = general_purpose::STANDARD.encode(&encrypted_data);

            let signed_sensor_data = TceSensorData {
                tceId: tce.tceId.clone(),
                sensorkey: public_key.to_pkcs1_pem(LineEnding::LF).unwrap(),
                signedSensorData: encrypted_data_base64,
                sensorData: sensor_data,
                commitment: general_purpose::STANDARD.encode(commitment),
                salt: salt_base64,
            };

            tces.push(tce);
            tocs.push(toc);
            ssd.push(signed_sensor_data);
        }

        for _ in 0..m {
            let hoc = self.generate_random_hoc();

            let tce: TCE = TCE {
                tceId: self.rng.gen_range(0..10000).to_string(),
                prevTceIds: tces.iter().map(|t| t.tceId.clone()).collect(),
                hocId: Some(hoc.hocId.clone()),
                tocId: None,
                shipmentId: shipment_id.clone(),
                mass: mass.clone(),
                co2eWTW: None,
                co2eTTW: None,
                transportActivity: None,
                distance: None,
            };

            tces.push(tce);
            hocs.push(hoc);
        }

        let footprint = ProductFootprint {
            id: self.rng.gen_range(0..10000).to_string(),
            specVersion: "None".to_string(),
            version: 0,
            created: "None".to_string(),
            status: "None".to_string(),
            companyName: "None".to_string(),
            companyIds: Vec::new(),
            productDescription: "None".to_string(),
            productIds: Vec::new(),
            productCategoryCpc: 0,
            productNameCompany: "None".to_string(),
            pcf: None,
            comment: "None".to_string(),
            extensions: vec![product_footprint::Extension {
                specVersion: "2.0.0".to_string(),
                dataSchema: "None".to_string(),
                data: product_footprint::ExtensionData {
                    mass,
                    shipmentId: shipment_id.clone(),
                    tces: tces.clone(),
                },
            }],
        };

        ProofingDocument {
            productFootprint: footprint,
            hocData: hocs,
            tocData: tocs,
            signedSensorData: Some(ssd),
            proof: Vec::new(),
        }
    }
}

fn create_numbered_file(base_path: &Path, extension: &str) -> io::Result<PathBuf> {
    let mut counter = 0;
    loop {
        let file_name = format!(
            "{}_{}.{}",
            base_path.file_name().unwrap().to_string_lossy(),
            counter,
            extension
        );

        let mut path = PathBuf::from(base_path.parent().unwrap_or_else(|| Path::new(".")));
        path.push(&file_name);

        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(_file) => {
                return Ok(path);
            }
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
                counter += 1;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
