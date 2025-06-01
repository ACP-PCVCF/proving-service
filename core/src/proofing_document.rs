#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use super::hoc_toc_data::{HocData, TocData};
use super::product_footprint::{ProductFootprint};
use super::sensor_data::{SensorData};

#[derive(Debug, Serialize, Deserialize)]
pub struct TceSensorData {
    pub tceId: String,
    pub sensorkey: String,
    pub signedSensorData: String,
    pub sensorData: SensorData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofingDocument {
    pub productFootprint: ProductFootprint,
    pub tocData: Vec<TocData>,
    pub hocData: Vec<HocData>,
    pub signedSensorData: Option<Vec<TceSensorData>>,
}