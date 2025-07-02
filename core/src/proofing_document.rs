#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use crate::product_footprint::ProductProof;

use super::hoc_toc_data::{HocData, TocData};
use super::product_footprint::{ProductFootprint, ProofExtension, Distance};

#[derive(Debug, Serialize, Deserialize)]
pub struct TceSensorData {
    pub tceId: String,
    pub sensorkey: String,
    pub signedSensorData: String,
    pub sensorData: SensorData,
    pub commitment: String,
    pub salt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofingDocument {
    pub productFootprint: ProductFootprint,
    pub tocData: Vec<TocData>,
    pub hocData: Vec<HocData>,
    pub signedSensorData: Option<Vec<TceSensorData>>,
    pub proof: Vec<ProductProof>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SensorData {
    pub distance: Distance,
}