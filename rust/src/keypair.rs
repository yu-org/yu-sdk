//! Key pair management for yu SDK (Rust).
//!
//! Supported key types:
//!   - Ed25519   via `ed25519-dalek`
//!   - Secp256k1 via `k256` (Tendermint-compatible)
//!
//! Key type index bytes (`yu/core/keypair` constants):
//!   Sr25519:   b'1' (0x31)
//!   Ed25519:   b'2' (0x32)
//!   Secp256k1: b'3' (0x33)
//!
//! Note: yu's own `SecpPubkey.BytesWithType()` prepends `Sr25519Idx` (b'1'),
//! which its `PubKeyFromBytes` would then decode back as an sr25519 key.
//! This SDK emits `Secp256k1Idx` (b'3') so the node decodes it as secp256k1.
//!
//! Address derivation (Tendermint):
//!   Ed25519 / Sr25519: SHA256(pubkey_bytes)[..20]
//!   Secp256k1:         RIPEMD160(SHA256(compressed_pubkey))

use std::fmt;

use ed25519_dalek::SigningKey as EdSigningKey;
use k256::ecdsa::signature::hazmat::PrehashSigner;
use k256::ecdsa::{Signature as SecpSignature, SigningKey as SecpSigningKey};
use k256::elliptic_curve::bigint::{Encoding, NonZero, U256};
use k256::elliptic_curve::Curve;
use rand::rngs::OsRng;
use rand::RngCore;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Ed25519,
    Sr25519,
    Secp256k1,
}

impl KeyType {
    /// The one-byte type index yu prepends to a key in its `*WithType` forms.
    pub fn idx(&self) -> u8 {
        match self {
            KeyType::Sr25519 => b'1',
            KeyType::Ed25519 => b'2',
            KeyType::Secp256k1 => b'3',
        }
    }

    /// The type name used by yu (`keypair.Ed25519`, `keypair.Secp256k1`, ...).
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyType::Sr25519 => "sr25519",
            KeyType::Ed25519 => "ed25519",
            KeyType::Secp256k1 => "secp256k1",
        }
    }
}

/// Errors returned when building a key pair from raw material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyError {
    /// The bytes are not a valid private key for the curve.
    InvalidPrivateKey,
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyError::InvalidPrivateKey => write!(f, "invalid private key bytes"),
        }
    }
}

impl std::error::Error for KeyError {}

fn to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

fn sha256_truncated(data: &[u8]) -> Vec<u8> {
    let hash = Sha256::digest(data);
    hash[..20].to_vec()
}

/// RIPEMD160(SHA256(data)) — Tendermint's secp256k1 address hash.
fn ripemd160_sha256(data: &[u8]) -> Vec<u8> {
    let sha = Sha256::digest(data);
    Ripemd160::digest(sha).to_vec()
}

// Both variants are boxed so the enum stays small: an ed25519-dalek
// SigningKey is a few hundred bytes, a k256 one only a few dozen.
enum Signer {
    Ed25519(Box<EdSigningKey>),
    Secp256k1(Box<SecpSigningKey>),
}

pub struct KeyPair {
    pub key_type: KeyType,
    signer: Signer,
}

impl KeyPair {
    // ---------- Ed25519 ----------

    /// Generate a new Ed25519 key pair.
    pub fn generate_ed25519() -> Self {
        Self {
            key_type: KeyType::Ed25519,
            signer: Signer::Ed25519(Box::new(EdSigningKey::generate(&mut OsRng))),
        }
    }

    /// Create from a 32-byte Ed25519 private key seed.
    pub fn from_ed25519_bytes(bytes: &[u8; 32]) -> Self {
        Self {
            key_type: KeyType::Ed25519,
            signer: Signer::Ed25519(Box::new(EdSigningKey::from_bytes(bytes))),
        }
    }

    // ---------- Secp256k1 ----------

    /// Generate a new Secp256k1 key pair, mirroring tendermint `GenPrivKey()`:
    /// random 32 bytes, retried until they form a valid scalar in `(0, n)`.
    pub fn generate_secp256k1() -> Self {
        loop {
            let mut bytes = [0u8; 32];
            OsRng.fill_bytes(&mut bytes);
            if let Ok(kp) = Self::from_secp256k1_bytes(&bytes) {
                return kp;
            }
        }
    }

    /// Create from a raw 32-byte Secp256k1 private key.
    ///
    /// Returns [`KeyError::InvalidPrivateKey`] if the bytes are zero or not
    /// below the curve order.
    pub fn from_secp256k1_bytes(bytes: &[u8; 32]) -> Result<Self, KeyError> {
        let signing_key =
            SecpSigningKey::from_slice(bytes).map_err(|_| KeyError::InvalidPrivateKey)?;
        Ok(Self {
            key_type: KeyType::Secp256k1,
            signer: Signer::Secp256k1(Box::new(signing_key)),
        })
    }

    /// Derive a Secp256k1 key pair from a secret, mirroring tendermint
    /// `GenPrivKeySecp256k1`: `k = (sha256(secret) mod (n - 1)) + 1`.
    ///
    /// This matches yu's `GenSecpKeyWithSecret`, so the same secret yields the
    /// same key in Go and in Rust.
    pub fn from_secp256k1_secret(secret: &[u8]) -> Self {
        let c = U256::from_be_slice(&Sha256::digest(secret));
        let n_minus_1 = k256::Secp256k1::ORDER.wrapping_sub(&U256::ONE);
        let modulus = NonZero::new(n_minus_1).unwrap();
        let k = c.rem(&modulus).wrapping_add(&U256::ONE);
        // k is in [1, n-1], so it is always a valid private key.
        Self::from_secp256k1_bytes(&k.to_be_bytes())
            .expect("secret-derived scalar is always in range")
    }

    // ---------- Common ----------

    /// Raw public key bytes: 32 bytes for Ed25519, 33-byte compressed point
    /// for Secp256k1.
    pub fn pubkey_bytes(&self) -> Vec<u8> {
        match &self.signer {
            Signer::Ed25519(sk) => sk.verifying_key().to_bytes().to_vec(),
            Signer::Secp256k1(sk) => sk.verifying_key().to_sec1_bytes().to_vec(),
        }
    }

    /// Raw private key bytes (32 bytes for both supported curves).
    pub fn privkey_bytes(&self) -> Vec<u8> {
        match &self.signer {
            Signer::Ed25519(sk) => sk.to_bytes().to_vec(),
            Signer::Secp256k1(sk) => sk.to_bytes().to_vec(),
        }
    }

    /// Returns "0x" + hex(type_prefix + pubkey_bytes) — matches StringWithType().
    pub fn pubkey_with_type(&self) -> String {
        let mut combined = vec![self.key_type.idx()];
        combined.extend_from_slice(&self.pubkey_bytes());
        to_hex(&combined)
    }

    /// Returns "0x" + hex(address) — matches Address.String() in yu.
    pub fn address(&self) -> String {
        let pubkey = self.pubkey_bytes();
        let addr = match self.key_type {
            KeyType::Ed25519 | KeyType::Sr25519 => sha256_truncated(&pubkey),
            KeyType::Secp256k1 => ripemd160_sha256(&pubkey),
        };
        to_hex(&addr)
    }

    /// Sign the given message bytes (32-byte hash from BytesToHash).
    ///
    /// Ed25519 signs the message directly (64-byte signature). Secp256k1
    /// follows tendermint: ECDSA over `sha256(msg)`, serialized as 64-byte
    /// `R || S` in lower-S form.
    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        match &self.signer {
            Signer::Ed25519(sk) => {
                use ed25519_dalek::Signer as _;
                sk.sign(msg).to_bytes().to_vec()
            }
            Signer::Secp256k1(sk) => {
                let digest = Sha256::digest(msg);
                let sig: SecpSignature = sk
                    .sign_prehash(&digest)
                    .expect("signing a 32-byte prehash cannot fail");
                let sig = sig.normalize_s().unwrap_or(sig);
                sig.to_bytes().to_vec()
            }
        }
    }
}
