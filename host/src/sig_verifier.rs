use base64::{ engine::general_purpose, Engine as _ };
use rand::rngs::OsRng;
use rsa::pkcs8::spki;
use rsa::RsaPrivateKey;
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey};
use spki::EncodePublicKey; // Import only EncodePublicKey for to_public_key_pem
use rsa::pkcs1v15::Pkcs1v15Sign;
use sha2::{Sha256, Digest as Sha2DigestTrait};
use const_oid::AssociatedOid;
use pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey as _, ObjectIdentifier};
use digest::{
    self,
    Digest as DigestTrait,
    OutputSizeUser,
    Reset,
    FixedOutputReset,
    generic_array::GenericArray,
    FixedOutput,
    Update
};

#[derive(Default, Clone)]
struct Sha256WithOid(Sha256);

impl AssociatedOid for Sha256WithOid {
    const OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");
}

impl OutputSizeUser for Sha256WithOid {
    type OutputSize = <Sha256 as OutputSizeUser>::OutputSize;
}

impl Update for Sha256WithOid {
    fn update(&mut self, data: &[u8]) {
        Update::update(&mut self.0, data);
    }
}

impl FixedOutput for Sha256WithOid {
    fn finalize_into(self, out: &mut GenericArray<u8, Self::OutputSize>) {
        FixedOutput::finalize_into(self.0, out);
    }
}

impl Reset for Sha256WithOid {
    fn reset(&mut self) {
        Reset::reset(&mut self.0);
    }
}

impl FixedOutputReset for Sha256WithOid {
     fn finalize_fixed_reset(&mut self) -> GenericArray<u8, Self::OutputSize> {
        FixedOutputReset::finalize_fixed_reset(&mut self.0)
     }
     fn finalize_into_reset(&mut self, out: &mut GenericArray<u8, Self::OutputSize>) {
        FixedOutputReset::finalize_into_reset(&mut self.0, out);
     }
}

impl DigestTrait for Sha256WithOid {
    fn new() -> Self {
        Sha256WithOid(Sha256::new())
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        Update::update(self, data.as_ref());
    }

    fn finalize(self) -> GenericArray<u8, Self::OutputSize> {
        DigestTrait::finalize(self.0)
    }

    fn new_with_prefix(data: impl AsRef<[u8]>) -> Self {
        Sha256WithOid(Sha256::new_with_prefix(data))
    }

    fn chain_update(self, data: impl AsRef<[u8]>) -> Self {
         Sha256WithOid(self.0.chain_update(data))
    }

    fn finalize_into(self, out: &mut GenericArray<u8, Self::OutputSize>) {
        DigestTrait::finalize_into(self.0, out);
    }

    fn finalize_reset(&mut self) -> GenericArray<u8, Self::OutputSize> {
        DigestTrait::finalize_reset(&mut self.0)
    }

    fn finalize_into_reset(&mut self, out: &mut GenericArray<u8, Self::OutputSize>) {
        DigestTrait::finalize_into_reset(&mut self.0, out);
    }

    fn reset(&mut self) {
        Reset::reset(&mut self.0);
    }

    fn output_size() -> usize {
        <Sha256 as DigestTrait>::output_size()
    }

    fn digest(data: impl AsRef<[u8]>) -> GenericArray<u8, Self::OutputSize> {
        <Sha256 as DigestTrait>::digest(data)
    }
}


pub fn verify_signature(commitment: &str, signed_sensor_data: &str, sensorkey: &str) -> bool {
    let payload = &commitment;
    let signature_b64 = &signed_sensor_data;
    let public_key_pem = &sensorkey;

    let public_key = match RsaPublicKey::from_public_key_pem(public_key_pem) {
        Ok(pk) => {
            println!("Signatur erfolgreich verifiziert");
            pk
        },
        Err(e) => {
            eprintln!("Fehler beim Laden des Public Keys (SPKI erwartet): {:?}", e);
            match RsaPublicKey::from_pkcs1_pem(public_key_pem) {
                Ok(pk_fallback) => {
                    eprintln!("Warnung: Public Key wurde als PKCS#1 geladen, SPKI wird bevorzugt.");
                    pk_fallback
                },
                Err(e_fallback) => {
                    eprintln!("Fehler beim Laden des Public Keys auch als PKCS#1: {:?}", e_fallback);
                    return false;
                }
            }
        }
    };

    let mut hasher = Sha256::new();
    Update::update(&mut hasher, payload.as_bytes());
    let digest_val = hasher.finalize();

    let signature = match general_purpose::STANDARD.decode(signature_b64) {
        Ok(sig) => sig,
        Err(e) => {
            eprintln!("Fehler beim Dekodieren der Signatur: {:?}", e);
            return false;
        }
    };

    let padding = Pkcs1v15Sign::new::<Sha256WithOid>();
    match public_key.verify(padding, &digest_val, &signature) {
        Ok(_) => {
            println!("Signatur ist gültig.");
            true
        }
        Err(e) => {
            eprintln!("Verifikation fehlgeschlagen: {:?}", e);
            false
        }
    }
}

pub fn hash(data: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    Update::update(&mut hasher, data.as_bytes());
    let computed_hash = hasher.finalize();
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&computed_hash);
    hash_array
}

pub fn generate_key_pair() -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut rng = OsRng;
    let bits = 2048; // Empfohlene Bitlänge für RSA

    let private_key = RsaPrivateKey::new(&mut rng, bits)?;
    let public_key = RsaPublicKey::from(&private_key);

    let private_key_pem: String = private_key.to_pkcs1_pem(pkcs1::LineEnding::LF)?.to_string();
    let public_key_pem: String = public_key.to_public_key_pem(pkcs1::LineEnding::LF)?;

    Ok((private_key_pem, public_key_pem))
}

pub fn sign_data(data: &str, private_key_pem: &str) -> Result<String, Box<dyn std::error::Error>> {
    let private_key = RsaPrivateKey::from_pkcs1_pem(private_key_pem)?;
    let mut hasher = Sha256::new();
    Update::update(&mut hasher, data.as_bytes());
    let digest_val = hasher.finalize();

    let padding = Pkcs1v15Sign::new::<Sha256WithOid>();
    let signature = private_key.sign(padding, &digest_val)?;

    Ok(general_purpose::STANDARD.encode(signature))
}
