// Key pair unit tests — no running chain required.
//
// The secp256k1 vectors below were produced by yu itself:
//
//   pub, priv, _ := keypair.GenKeyPairWithSecret(keypair.Secp256k1, secret)
//   sig, _ := priv.SignData(msg)
//
// with secret = "yu-sdk-secp256k1-test-secret" and msg = [0x00, 0x01, ..., 0x1f].

use yu_sdk::{KeyPair, KeyType};

const SECRET: &[u8] = b"yu-sdk-secp256k1-test-secret";

const GO_PRIVKEY: &str = "61b491f9ffd837c36da7ca7bd921fdfb66ff37951fda43d7ec29455e0014790b";
const GO_PUBKEY: &str = "024a3bb54e93dd3bb3c0a54611c6f5d34e3a01b7f0f638230378b8c9ad238838d3";
const GO_ADDRESS: &str = "0x4a74ffa853c4226b8133702d0f9ee09cc1e12b9e";
const GO_SIGNATURE: &str = "6d05fadb1d4e6c391b90af818d5cc2f8d37df0a96108262c9f15d2891c0e0c9f\
                            4709a9285cf460d9690a41a56d352c6c7ef6ec9673e106b886ad501f86011295";

fn test_msg() -> Vec<u8> {
    (0u8..32).collect()
}

#[test]
fn secp256k1_secret_derivation_matches_go() {
    let kp = KeyPair::from_secp256k1_secret(SECRET);
    assert_eq!(kp.key_type, KeyType::Secp256k1);
    assert_eq!(hex::encode(kp.privkey_bytes()), GO_PRIVKEY);
}

#[test]
fn secp256k1_pubkey_is_compressed_and_matches_go() {
    let kp = KeyPair::from_secp256k1_secret(SECRET);
    let pubkey = kp.pubkey_bytes();
    assert_eq!(pubkey.len(), 33);
    assert_eq!(hex::encode(&pubkey), GO_PUBKEY);
}

#[test]
fn secp256k1_address_matches_go() {
    let kp = KeyPair::from_secp256k1_secret(SECRET);
    assert_eq!(kp.address(), GO_ADDRESS);
}

#[test]
fn secp256k1_signature_matches_go() {
    let kp = KeyPair::from_secp256k1_secret(SECRET);
    let sig = kp.sign(&test_msg());
    // Tendermint serializes as 64-byte R || S in lower-S form; RFC6979 makes
    // the nonce deterministic, so the bytes match Go exactly.
    assert_eq!(sig.len(), 64);
    assert_eq!(hex::encode(sig), GO_SIGNATURE);
}

#[test]
fn secp256k1_pubkey_with_type_uses_secp_idx() {
    let kp = KeyPair::from_secp256k1_secret(SECRET);
    // b'3' == 0x33 == keypair.Secp256k1Idx, what yu's PubKeyFromBytes dispatches on.
    assert_eq!(kp.pubkey_with_type(), format!("0x33{}", GO_PUBKEY));
}

#[test]
fn secp256k1_from_bytes_roundtrip() {
    let kp = KeyPair::from_secp256k1_secret(SECRET);
    let raw: [u8; 32] = kp.privkey_bytes().try_into().unwrap();
    let restored = KeyPair::from_secp256k1_bytes(&raw).expect("valid private key");
    assert_eq!(restored.pubkey_bytes(), kp.pubkey_bytes());
    assert_eq!(restored.address(), kp.address());
}

#[test]
fn secp256k1_rejects_invalid_private_key() {
    assert!(KeyPair::from_secp256k1_bytes(&[0u8; 32]).is_err());
    // n itself is out of range (valid keys are in [1, n-1]).
    let order = hex::decode("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141")
        .unwrap();
    let order: [u8; 32] = order.try_into().unwrap();
    assert!(KeyPair::from_secp256k1_bytes(&order).is_err());
}

#[test]
fn secp256k1_generate_produces_distinct_usable_keys() {
    let a = KeyPair::generate_secp256k1();
    let b = KeyPair::generate_secp256k1();
    assert_eq!(a.key_type, KeyType::Secp256k1);
    assert_ne!(a.privkey_bytes(), b.privkey_bytes());
    assert_eq!(a.sign(&test_msg()).len(), 64);
}

#[test]
fn ed25519_still_uses_sha256_address_and_idx_2() {
    let kp = KeyPair::from_ed25519_bytes(&[7u8; 32]);
    assert_eq!(kp.key_type, KeyType::Ed25519);
    assert_eq!(kp.pubkey_bytes().len(), 32);
    assert_eq!(kp.sign(&test_msg()).len(), 64);
    // 0x + b'2' + 32-byte pubkey
    assert!(kp.pubkey_with_type().starts_with("0x32"));
    // SHA256(pubkey)[..20] -> 20 bytes -> 40 hex chars
    assert_eq!(kp.address().len(), 42);
}
