#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureContainer {
    pub tceId: String,
    pub commitment: String,
    pub salt: String,
    pub sensorData: String,
}