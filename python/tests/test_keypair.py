"""
Key pair unit tests — no running chain required.

The secp256k1 vectors below were produced by yu itself:

    pub, priv, _ := keypair.GenKeyPairWithSecret(keypair.Secp256k1, secret)
    sig, _ := priv.SignData(msg)

with secret = b"yu-sdk-secp256k1-test-secret" and msg = bytes(range(32)).
These are the same vectors used by the Rust SDK's tests/keypair_test.rs.
"""

import pytest

from yu_sdk.keypair import KeyPair, KeyType

SECRET = b"yu-sdk-secp256k1-test-secret"

GO_PRIVKEY = "61b491f9ffd837c36da7ca7bd921fdfb66ff37951fda43d7ec29455e0014790b"
GO_PUBKEY = "024a3bb54e93dd3bb3c0a54611c6f5d34e3a01b7f0f638230378b8c9ad238838d3"
GO_ADDRESS = "0x4a74ffa853c4226b8133702d0f9ee09cc1e12b9e"
GO_SIGNATURE = (
    "6d05fadb1d4e6c391b90af818d5cc2f8d37df0a96108262c9f15d2891c0e0c9f"
    "4709a9285cf460d9690a41a56d352c6c7ef6ec9673e106b886ad501f86011295"
)


def _test_msg() -> bytes:
    return bytes(range(32))


def test_secp256k1_secret_derivation_matches_go():
    kp = KeyPair.from_secret(KeyType.SECP256K1, SECRET)
    assert kp.key_type == KeyType.SECP256K1
    assert kp.privkey_bytes.hex() == GO_PRIVKEY


def test_secp256k1_pubkey_is_compressed_and_matches_go():
    kp = KeyPair.from_secret(KeyType.SECP256K1, SECRET)
    assert len(kp.pubkey_bytes) == 33
    assert kp.pubkey_bytes.hex() == GO_PUBKEY


def test_secp256k1_address_matches_go():
    kp = KeyPair.from_secret(KeyType.SECP256K1, SECRET)
    assert kp.address == GO_ADDRESS


def test_secp256k1_signature_matches_go():
    kp = KeyPair.from_secret(KeyType.SECP256K1, SECRET)
    sig = kp.sign(_test_msg())
    # Tendermint serializes as 64-byte R || S in lower-S form; RFC6979 makes
    # the nonce deterministic, so the bytes match Go exactly.
    assert len(sig) == 64
    assert sig.hex() == GO_SIGNATURE


def test_secp256k1_pubkey_with_type_uses_secp_idx():
    kp = KeyPair.from_secret(KeyType.SECP256K1, SECRET)
    # 0x33 == keypair.Secp256k1Idx, what yu's PubKeyFromBytes dispatches on.
    assert kp.pubkey_with_type == f"0x33{GO_PUBKEY}"


def test_secp256k1_from_bytes_roundtrip():
    kp = KeyPair.from_secret(KeyType.SECP256K1, SECRET)
    restored = KeyPair.from_private_bytes(KeyType.SECP256K1, kp.privkey_bytes)
    assert restored.pubkey_bytes == kp.pubkey_bytes
    assert restored.address == kp.address


def test_secp256k1_rejects_invalid_private_key():
    with pytest.raises(ValueError):
        KeyPair.from_private_bytes(KeyType.SECP256K1, bytes(32))
    # n itself is out of range (valid keys are in [1, n-1]).
    order = bytes.fromhex(
        "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141"
    )
    with pytest.raises(ValueError):
        KeyPair.from_private_bytes(KeyType.SECP256K1, order)


def test_secp256k1_generate_produces_distinct_usable_keys():
    a = KeyPair(KeyType.SECP256K1)
    b = KeyPair(KeyType.SECP256K1)
    assert a.key_type == KeyType.SECP256K1
    assert a.privkey_bytes != b.privkey_bytes
    assert len(a.sign(_test_msg())) == 64


def test_ed25519_still_uses_sha256_address_and_idx_2():
    kp = KeyPair.from_private_bytes(KeyType.ED25519, bytes([7] * 32))
    assert kp.key_type == KeyType.ED25519
    assert len(kp.pubkey_bytes) == 32
    assert len(kp.sign(_test_msg())) == 64
    assert kp.pubkey_with_type.startswith("0x32")
    # SHA256(pubkey)[:20] -> 20 bytes -> 40 hex chars (+ "0x")
    assert len(kp.address) == 42
