/**
 * Key pair management for yu SDK (JavaScript).
 *
 * Supports Ed25519 (default).
 * Sr25519 support requires @polkadot/util-crypto (optional).
 *
 * Key type index bytes (matches yu/core/keypair constants):
 *   Sr25519:  0x31 (ASCII '1')
 *   Ed25519:  0x32 (ASCII '2')
 *   Secp256k1: 0x31 (same — matches yu secp256k1.go)
 *
 * Address derivation (Tendermint):
 *   Ed25519 / Sr25519: SHA256(pubkey)[:20]
 */

import * as ed from "@noble/ed25519";
import { sha256 } from "@noble/hashes/sha256";

export const KeyType = Object.freeze({
  ED25519: "ed25519",
  SR25519: "sr25519",
});

const KEY_TYPE_PREFIX = {
  [KeyType.SR25519]: 0x31,
  [KeyType.ED25519]: 0x32,
};

export function toHex(bytes) {
  return "0x" + Buffer.from(bytes).toString("hex");
}

function sha256Truncated(pubkeyBytes) {
  return sha256(pubkeyBytes).slice(0, 20);
}

export class KeyPair {
  /**
   * @param {string} keyType - KeyType.ED25519 (default) or KeyType.SR25519
   * @param {Uint8Array|null} privkeyBytes - optional 32-byte private key seed
   */
  constructor(keyType = KeyType.ED25519, privkeyBytes = null) {
    this.keyType = keyType;
    if (keyType === KeyType.ED25519) {
      this._privkey = privkeyBytes ?? ed.utils.randomPrivateKey();
    } else {
      throw new Error(
        `Key type ${keyType} not directly constructible. Use KeyPair.generateSr25519().`
      );
    }
    this._pubkeyBytes = null;
  }

  /** Generate an Sr25519 key pair using @polkadot/util-crypto. */
  static async generateSr25519() {
    const { sr25519PairFromSeed } = await import("@polkadot/util-crypto");
    const { randomAsU8a } = await import("@polkadot/util");
    const seed = randomAsU8a(32);
    const pair = sr25519PairFromSeed(seed);
    const kp = Object.create(KeyPair.prototype);
    kp.keyType = KeyType.SR25519;
    kp._privkey = pair.secretKey;
    kp._pubkeyBytes = pair.publicKey;
    return kp;
  }

  async getPubkeyBytes() {
    if (!this._pubkeyBytes) {
      if (this.keyType === KeyType.ED25519) {
        this._pubkeyBytes = await ed.getPublicKeyAsync(this._privkey);
      }
    }
    return this._pubkeyBytes;
  }

  async pubkeyWithType() {
    const pubBytes = await this.getPubkeyBytes();
    const prefix = KEY_TYPE_PREFIX[this.keyType];
    const combined = new Uint8Array(1 + pubBytes.length);
    combined[0] = prefix;
    combined.set(pubBytes, 1);
    return toHex(combined);
  }

  async getAddress() {
    const pubBytes = await this.getPubkeyBytes();
    const addrBytes = sha256Truncated(pubBytes);
    return toHex(addrBytes);
  }

  async sign(msgBytes) {
    if (this.keyType === KeyType.ED25519) {
      return ed.signAsync(msgBytes, this._privkey);
    }
    if (this.keyType === KeyType.SR25519) {
      const { sr25519Sign } = await import("@polkadot/util-crypto");
      const pubBytes = await this.getPubkeyBytes();
      return sr25519Sign(pubBytes, { secretKey: this._privkey, publicKey: pubBytes }, msgBytes);
    }
    throw new Error(`Signing not supported for ${this.keyType}`);
  }
}
