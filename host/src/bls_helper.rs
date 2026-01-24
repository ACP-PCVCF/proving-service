use blst::min_pk::{AggregateSignature, PublicKey, SecretKey, Signature};

// Domain Separation Tag - must match the one used in guest and by signers.
const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

/// Aggregates multiple BLS signatures into a single signature.
pub fn aggregate_signatures(signatures: &[Vec<u8>]) -> Vec<u8> {
    let sigs: Vec<Signature> = signatures
        .iter()
        .map(|s| Signature::from_bytes(s).expect("Invalid signature"))
        .collect();

    let sig_refs: Vec<&Signature> = sigs.iter().collect();

    let agg = AggregateSignature::aggregate(&sig_refs, true).expect("Aggregation failed");

    agg.to_signature().to_bytes().to_vec()
}

/// Signs a message using BLS
pub fn bls_sign(message: &[u8], secret_key: &[u8]) -> Vec<u8> {
    let sk = SecretKey::from_bytes(secret_key).expect("Invalid secret key");
    let sig = sk.sign(message, DST, &[]);
    sig.to_bytes().to_vec()
}

/// Verifies a single BLS signature
pub fn bls_verify(message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
    let pk = match PublicKey::from_bytes(public_key) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    let sig = match Signature::from_bytes(signature) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    sig.verify(true, message, DST, &[], &pk, true) == blst::BLST_ERROR::BLST_SUCCESS
}

/// Generates a BLS keypair.
pub fn generate_bls_keypair() -> (Vec<u8>, Vec<u8>) {
    let mut ikm = [0u8; 32];
    getrandom::getrandom(&mut ikm).unwrap();

    let sk = SecretKey::key_gen(&ikm, &[]).expect("Key generation failed");
    let pk = sk.sk_to_pk();

    (sk.to_bytes().to_vec(), pk.to_bytes().to_vec())
}
