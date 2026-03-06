/**
 * Key pair management for yu SDK (Rust).
 *
 * Supports Ed25519 via ed25519-dalek.
 *
 * Key type index bytes (matches yu/core/keypair constants):
 *   Sr25519:  b'1'  (0x31)
 *   Ed25519:  b'2'  (0x32)
 *   Secp256k1: b'1' (0x31, same — matches yu secp256k1.go)
 *
 * Address derivation (Tendermint):
 *   Ed25519 / Sr25519: SHA256(pubkey_bytes)[..20]
 */

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Ed25519,
    Sr25519,
}

fn to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

fn sha256_truncated(data: &[u8]) -> Vec<u8> {
    let hash = Sha256::digest(data);
    hash[..20].to_vec()
}

pub struct KeyPair {
    pub key_type: KeyType,
    signing_key: SigningKey,
}

impl KeyPair {
    /// Generate a new Ed25519 key pair.
    pub fn generate_ed25519() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self {
            key_type: KeyType::Ed25519,
            signing_key,
        }
    }

    /// Create from a 32-byte Ed25519 private key seed.
    pub fn from_ed25519_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        Self {
            key_type: KeyType::Ed25519,
            signing_key,
        }
    }

    pub fn pubkey_bytes(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
    }

    /// Returns "0x" + hex(type_prefix + pubkey_bytes) — matches StringWithType().
    pub fn pubkey_with_type(&self) -> String {
        let prefix: u8 = match self.key_type {
            KeyType::Ed25519 => 0x32,  // b'2'
            KeyType::Sr25519 => 0x31,  // b'1'
        };
        let mut combined = vec![prefix];
        combined.extend_from_slice(&self.pubkey_bytes());
        to_hex(&combined)
    }

    /// Returns "0x" + hex(address) — matches Address.String() in yu.
    pub fn address(&self) -> String {
        let addr = sha256_truncated(&self.pubkey_bytes());
        to_hex(&addr)
    }

    /// Sign the given message bytes (32-byte hash from BytesToHash).
    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        use ed25519_dalek::Signer;
        let sig = self.signing_key.sign(msg);
        sig.to_bytes().to_vec()
    }
}
