#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum CertificationEnum {
    #[serde(rename = "ISO14083:2023")]
    Iso14083_2023,
    #[serde(rename = "ISO_14001")]
    Iso14001,
    #[serde(rename = "ECO_TRANSIT_CERT")]
    EcoTransitCert,
    #[serde(rename = "GLECv2")]
    GleCv2,
    #[serde(rename = "GLECv3")]
    GleCv3,
    #[serde(rename = "GLECv3.1")]
    GleCv3_1,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransportMode {
    #[serde(rename = "road")]
    Road,
    #[serde(rename = "air")]
    Air,
    #[serde(rename = "sea")]
    Sea,
    #[serde(rename = "rail")]
    Rail,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct EnergyCarriers {
    pub energyCarrier: String,
    pub relativeShare: String,
    pub emissionFactorWTW: String,
    pub emissionFactorTTW: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TocData {
    pub tocId: String,
    pub certifications: Vec<CertificationEnum>,
    pub description: String,
    pub mode: TransportMode,
    pub loadFactor: String,
    pub emptyDistanceFactor: String,
    pub temperatureControl: String,
    pub truckLoadingSequence: String,
    pub airShippingOption: Option<String>,
    pub flightLength: Option<String>,
    pub energyCarriers: Vec<EnergyCarriers>,
    pub co2eIntensityWTW: String,
    pub co2eIntensityTTW: String,
    pub transportActivityUnit: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct HocData {
    pub hocId: String,
    pub passhubType: String,
    pub energyCarriers: Vec<EnergyCarriers>,
    pub co2eIntensityWTW: String,
    pub co2eIntensityTTW: String,
    pub hubActivityUnit: String,
}