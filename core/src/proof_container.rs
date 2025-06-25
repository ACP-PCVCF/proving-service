#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use risc0_zkvm::{sha::Digest, Journal};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofContainer {
    pub image_id: Digest,
    pub journal: Journal,
}