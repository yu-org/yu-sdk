/**
 * YuClient: HTTP + WebSocket client for the yu blockchain.
 *
 * Writing endpoint: POST <httpUrl>/api/writing
 * Reading endpoint: POST <httpUrl>/api/reading
 * Event stream:     WS   <wsUrl>/subscribe/results
 */

import WebSocket from "ws";
import { toHex } from "./keypair.js";

/**
 * Replicates yu common.BytesToHash:
 * Takes the last 32 bytes of data (left-pads with zeros if < 32 bytes).
 * @param {Uint8Array|Buffer} data
 * @returns {Uint8Array} 32-byte hash
 */
function bytesToHash(data) {
  if (data.length >= 32) {
    return data.slice(data.length - 32);
  }
  const padded = new Uint8Array(32);
  padded.set(data, 32 - data.length);
  return padded;
}

async function post(url, body) {
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${await response.text()}`);
  }
  return response.json();
}

export class YuClient {
  /**
   * @param {string} httpUrl - e.g. "http://localhost:7999"
   * @param {string} wsUrl   - e.g. "ws://localhost:8999"
   */
  constructor(httpUrl = "http://localhost:7999", wsUrl = "ws://localhost:8999") {
    this.httpUrl = httpUrl.replace(/\/$/, "");
    this.wsUrl = wsUrl.replace(/\/$/, "");
    this._keypair = null;
  }

  withKeypair(keypair) {
    this._keypair = keypair;
    return this;
  }

  /**
   * Send a signed writing transaction to the chain.
   * @param {string} tripodName
   * @param {string} funcName
   * @param {object} params
   * @param {number} leiPrice
   * @param {number} tips
   */
  async writeChain(tripodName, funcName, params, leiPrice = 0, tips = 0) {
    if (!this._keypair) throw new Error("KeyPair not set; call withKeypair() first");

    const wrCall = {
      tripod_name: tripodName,
      func_name: funcName,
      params: JSON.stringify(params),
    };
    if (leiPrice) wrCall.lei_price = leiPrice;
    if (tips) wrCall.tips = tips;

    // Sign: BytesToHash(JSON.stringify(wrCall))
    const callJson = new TextEncoder().encode(JSON.stringify(wrCall));
    const msgHash = bytesToHash(callJson);
    const sig = await this._keypair.sign(msgHash);

    const postBody = {
      pubkey: await this._keypair.pubkeyWithType(),
      address: await this._keypair.getAddress(),
      signature: toHex(sig),
      call: wrCall,
    };

    await post(`${this.httpUrl}/api/writing`, postBody);
  }

  /**
   * Send a reading query to the chain.
   * @param {string} tripodName
   * @param {string} funcName
   * @param {object} params
   * @returns {Promise<any>} parsed JSON response
   */
  async readChain(tripodName, funcName, params) {
    const rdCall = {
      tripod_name: tripodName,
      func_name: funcName,
      params: JSON.stringify(params),
    };
    return post(`${this.httpUrl}/api/reading`, rdCall);
  }

  /**
   * Subscribe to chain events.
   * @param {function(object): void} callback - called for each receipt
   * @returns {EventSubscriber}
   */
  subscribeEvents(callback) {
    const sub = new EventSubscriber(`${this.wsUrl}/subscribe/results`, callback);
    sub.connect();
    return sub;
  }

  /** Send admin stop request. */
  async stopChain() {
    try {
      await fetch(`${this.httpUrl}/api/admin/stop`);
    } catch (_) {}
  }
}

export class EventSubscriber {
  constructor(url, callback) {
    this._url = url;
    this._callback = callback;
    this._ws = null;
  }

  connect() {
    this._ws = new WebSocket(this._url);
    this._ws.on("message", (data) => {
      try {
        const receipt = JSON.parse(data.toString());
        this._callback(receipt);
      } catch (_) {}
    });
    this._ws.on("error", () => {});
  }

  close() {
    if (this._ws) this._ws.close();
  }
}
