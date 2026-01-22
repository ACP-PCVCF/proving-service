#![allow(non_snake_case)]

use risc0_zkvm::{sha::Digest, Journal};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofContainer {
    pub image_id: Digest,
    pub journal: Journal,
}
