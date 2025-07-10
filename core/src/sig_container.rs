#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureContainer {
    pub commitment: String,
    pub signature: String,
    pub pub_key: String,
}