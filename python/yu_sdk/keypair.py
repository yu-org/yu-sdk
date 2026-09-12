"""
Key pair management for yu SDK.

Supports Ed25519 (default), Sr25519, and Secp256k1.
Address derivation follows Tendermint conventions:
  - Ed25519 / Sr25519: SHA256(pubkey_bytes)[:20]
  - Secp256k1:         RIPEMD160(SHA256(pubkey_bytes))

Key type index prefixes (ASCII char bytes, matches yu/core/keypair constants):
  - Sr25519:   b'1'  (0x31)
  - Ed25519:   b'2'  (0x32)
  - Secp256k1: b'3'  (0x33)
"""

import hashlib
import os
from enum import Enum

from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey,
)
from cryptography.hazmat.primitives.serialization import (
    Encoding,
    PublicFormat,
    PrivateFormat,
    NoEncryption,
)


class KeyType(Enum):
    ED25519 = "ed25519"
    SR25519 = "sr25519"
    SECP256K1 = "secp256k1"


# Key type index bytes (matches yu/core/keypair constants)
_KEY_TYPE_IDX = {
    KeyType.SR25519: b"1",
    KeyType.ED25519: b"2",
    KeyType.SECP256K1: b"3",
}

# secp256k1 curve order (n)
_SECP256K1_N = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141


def _to_hex(data: bytes) -> str:
    """Returns 0x-prefixed hex string, matching yu's ToHex / hexutil.Encode."""
    return "0x" + data.hex()


def _sha256_truncated(data: bytes) -> bytes:
    """SHA256(data)[:20] — Tendermint address hash for ed25519/sr25519."""
    return hashlib.sha256(data).digest()[:20]


class KeyPair:
    """A keypair that can sign writing transactions for the yu chain."""

    def __init__(self, key_type: KeyType = KeyType.ED25519):
        self.key_type = key_type
        if key_type == KeyType.ED25519:
            self._privkey = Ed25519PrivateKey.generate()
            self._pubkey_raw = self._privkey.public_key().public_bytes(
                Encoding.Raw, PublicFormat.Raw
            )
        elif key_type == KeyType.SR25519:
            self._privkey, self._pubkey_raw = _gen_sr25519()
        elif key_type == KeyType.SECP256K1:
            self._privkey, self._pubkey_raw = _gen_secp256k1()
        else:
            raise ValueError(f"Unsupported key type: {key_type}")

    @classmethod
    def from_private_bytes(cls, key_type: KeyType, priv_bytes: bytes) -> "KeyPair":
        kp = cls.__new__(cls)
        kp.key_type = key_type
        if key_type == KeyType.ED25519:
            kp._privkey = Ed25519PrivateKey.from_private_bytes(priv_bytes)
            kp._pubkey_raw = kp._privkey.public_key().public_bytes(
                Encoding.Raw, PublicFormat.Raw
            )
        elif key_type == KeyType.SECP256K1:
            kp._privkey, kp._pubkey_raw = _secp256k1_from_bytes(priv_bytes)
        else:
            raise NotImplementedError(f"from_private_bytes not supported for {key_type}")
        return kp

    @classmethod
    def from_secret(cls, key_type: KeyType, secret: bytes) -> "KeyPair":
        """Derive a key pair from an arbitrary-length secret.

        For Secp256k1 this matches yu's `GenSecpKeyWithSecret` /
        tendermint's `GenPrivKeySecp256k1`:
            k = (sha256(secret) mod (n - 1)) + 1
        so the same secret yields the same key pair as the Go and Rust SDKs.
        """
        kp = cls.__new__(cls)
        kp.key_type = key_type
        if key_type == KeyType.SECP256K1:
            sec_hash = hashlib.sha256(secret).digest()
            k = (int.from_bytes(sec_hash, "big") % (_SECP256K1_N - 1)) + 1
            priv_bytes = k.to_bytes(32, "big")
            kp._privkey, kp._pubkey_raw = _secp256k1_from_bytes(priv_bytes)
        else:
            raise NotImplementedError(f"from_secret not supported for {key_type}")
        return kp

    @property
    def pubkey_bytes(self) -> bytes:
        return self._pubkey_raw

    @property
    def privkey_bytes(self) -> bytes:
        if self.key_type == KeyType.ED25519:
            return self._privkey.private_bytes(
                Encoding.Raw, PrivateFormat.Raw, NoEncryption()
            )
        elif self.key_type == KeyType.SECP256K1:
            return self._privkey.to_bytes()
        raise NotImplementedError(f"privkey_bytes not supported for {self.key_type}")

    @property
    def pubkey_with_type(self) -> str:
        """Returns hex(type_prefix + pubkey_bytes) — matches StringWithType()."""
        prefix = _KEY_TYPE_IDX[self.key_type]
        return _to_hex(prefix + self._pubkey_raw)

    @property
    def address(self) -> str:
        """Returns 0x-prefixed hex address, matches yu Address.String()."""
        if self.key_type in (KeyType.ED25519, KeyType.SR25519):
            addr_bytes = _sha256_truncated(self._pubkey_raw)
        elif self.key_type == KeyType.SECP256K1:
            addr_bytes = _ripemd160_sha256(self._pubkey_raw)
        else:
            raise ValueError(f"Unsupported key type: {self.key_type}")
        return _to_hex(addr_bytes)

    def sign(self, msg: bytes) -> bytes:
        """Sign msg (32 bytes — BytesToHash result) and return signature bytes."""
        if self.key_type == KeyType.ED25519:
            return self._privkey.sign(msg)
        elif self.key_type == KeyType.SR25519:
            return _sr25519_sign(self._privkey, msg)
        elif self.key_type == KeyType.SECP256K1:
            return _secp256k1_sign(self._privkey, msg)
        else:
            raise ValueError(f"Unsupported key type: {self.key_type}")

    @property
    def signature_hex(self) -> str:
        """Helper — not used directly; call sign() and convert."""
        raise NotImplementedError


def _ripemd160_sha256(data: bytes) -> bytes:
    sha = hashlib.sha256(data).digest()
    h = hashlib.new("ripemd160")
    h.update(sha)
    return h.digest()


# ---------- Sr25519 ----------

def _gen_sr25519():
    """Generate sr25519 key pair using py-sr25519-bindings if available,
    otherwise fall back to a random ed25519-like stub for testing."""
    try:
        import sr25519
        seed = os.urandom(32)
        pub, priv = sr25519.pair_from_seed(seed)
        return priv, pub
    except ImportError:
        raise ImportError(
            "sr25519 support requires 'py-sr25519-bindings'.\n"
            "Install with: pip install py-sr25519-bindings"
        )


def _sr25519_sign(priv, msg: bytes) -> bytes:
    try:
        import sr25519
        pub = priv[32:]  # sr25519 priv is 64 bytes (priv_seed + pub)
        return sr25519.sign(pub, priv, msg)
    except ImportError:
        raise ImportError("sr25519 support requires 'py-sr25519-bindings'")


# ---------- Secp256k1 ----------
#
# yu / tendermint conventions this section mirrors:
#   - GenPrivKey(): 32 random bytes, resampled until 0 < d < n
#   - PrivKey.Sign(msg): ECDSA over SHA256(msg), returned as 64-byte R || S
#     in lower-S form (see tendermint/crypto/secp256k1/secp256k1.go)
#   - PubKey.Address(): RIPEMD160(SHA256(compressed_pubkey))

def _require_eth_keys():
    try:
        from eth_keys import keys as eth_keys
        return eth_keys
    except ImportError:
        raise ImportError(
            "secp256k1 support requires 'eth-keys'.\n"
            "Install with: pip install eth-keys"
        )


def _secp256k1_from_bytes(priv_bytes: bytes):
    eth_keys = _require_eth_keys()
    d = int.from_bytes(priv_bytes, "big")
    if not (0 < d < _SECP256K1_N):
        raise ValueError("invalid secp256k1 private key")
    priv = eth_keys.PrivateKey(priv_bytes)
    pub_bytes = priv.public_key.to_compressed_bytes()
    return priv, pub_bytes


def _gen_secp256k1():
    # Mirrors tendermint's GenPrivKey(): resample until the bytes form a
    # valid scalar (0 < d < n). A single 32-byte draw is virtually always
    # valid; the loop only protects the astronomically unlikely edge case.
    while True:
        priv_bytes = os.urandom(32)
        d = int.from_bytes(priv_bytes, "big")
        if 0 < d < _SECP256K1_N:
            return _secp256k1_from_bytes(priv_bytes)


def _secp256k1_sign(priv, msg: bytes) -> bytes:
    # tendermint signs SHA256(msg), not msg directly — msg here is already
    # the 32-byte BytesToHash result, so it still needs this extra hash.
    digest = hashlib.sha256(msg).digest()
    sig = priv.sign_msg_hash(digest)
    # eth_keys returns 65 bytes (r || s || v); yu/tendermint wants the
    # 64-byte r || s form with no recovery id.
    return sig.r.to_bytes(32, "big") + sig.s.to_bytes(32, "big")
