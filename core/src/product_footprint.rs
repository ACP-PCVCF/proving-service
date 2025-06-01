#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Distance {
    pub actual: Option<f64>,
    pub gcd: Option<f64>,
    pub sfd: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TCE {
    pub tceId: String,
    #[serde(default)]
    pub prevTceIds: Vec<String>,
    pub hocId: Option<String>,
    pub tocId: Option<String>,
    pub shipmentId: String,
    pub mass: f64,
    pub co2eWTW: Option<f64>,
    pub co2eTTW: Option<f64>,
    pub transportActivity: Option<f64>,
    pub distance: Option<Distance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtensionData {
    pub mass: f64,
    pub shipmentId: String,
    #[serde(default)]
    pub tces: Vec<TCE>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Extension {
    #[serde(default = "default_spec_version")]
    pub specVersion: String,
    pub dataSchema: String,
    pub data: ExtensionData,
}

fn default_spec_version() -> String {
    "2.0.0".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductFootprint {
    pub id: String,
    #[serde(default = "default_spec_version")]
    pub specVersion: String,
    #[serde(default)]
    pub version: i32,
    pub created: String,
    #[serde(default = "default_status")]
    pub status: String,
    pub companyName: String,
    pub companyIds: Vec<String>,
    pub productDescription: String,
    pub productIds: Vec<String>,
    pub productCategoryCpc: i32,
    pub productNameCompany: String,
    pub pcf: Option<f64>,
    #[serde(default = "default_comment")]
    pub comment: String,
    #[serde(default)]
    pub extensions: Vec<Extension>,
}

fn default_status() -> String {
    "Active".to_string()
}

fn default_comment() -> String {
    "".to_string()
}