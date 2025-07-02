#![allow(non_snake_case)]
use super::proofing_document::SensorData;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureContainer {
    // pub tceId: String,
    pub commitment: String,
    // pub salt: String,
    // pub sensorData: String,
    pub signature: String,
    pub pub_key: String,
}

pub struct GuestCommunication {
    pub pcf_value: f64,
    pub encoded_sig_containers: String,
}