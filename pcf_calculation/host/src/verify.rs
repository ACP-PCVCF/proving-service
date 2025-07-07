use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
//use hex;
use methods::GUEST_CODE_FOR_ZK_PROOF_ID;
use risc0_zkvm::{Digest, Receipt};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct ReceiptExport {
    image_id: String,
    receipt: Receipt,
}

#[derive(Deserialize)]
struct ReceiptExportJson {
    image_id: String,
    receipt: String,
}

pub fn verify_receipt() -> Result<()> {
    // 1. JSON-Datei lesen
    let receipt_json_str = fs::read_to_string("receipt_output.json")
        .with_context(|| format!("Konnte Datei '{}' nicht lesen", "receipt_output.json"))?;

    let export: ReceiptExportJson = serde_json::from_str(&receipt_json_str)
        .context("Deserialisierung des Receipts fehlgeschlagen")?;

    let receipt_bytes = general_purpose::STANDARD
        .decode(&export.receipt)
        .context("Base64-Dekodierung fehlgeschlagen")?;

    let receipt: Receipt = bincode::deserialize(&receipt_bytes)
        .context("Bincode-Deserialisierung des Receipts fehlgeschlagen")?;

    receipt.verify(GUEST_CODE_FOR_ZK_PROOF_ID)?;

    println!("✅ Receipt erfolgreich aus Datei geladen und verifiziert!");

    Ok(())
}