"""
YuClient: HTTP + WebSocket client for the yu blockchain.

Writing endpoint: POST http://<host>/api/writing
Reading endpoint: POST http://<host>/api/reading
Event stream:     WS   ws://<host>/subscribe/results
"""

import hashlib
import json
import threading
from typing import Any, Callable, Optional

import requests
import websocket  # websocket-client

from .keypair import KeyPair, _to_hex
from .types import RdCall, Receipt, WrCall


def _bytes_to_hash(data: bytes) -> bytes:
    """
    Replicates yu common.BytesToHash: takes the last 32 bytes of data
    (left-pads with zeros if shorter than 32 bytes).
    """
    if len(data) >= 32:
        return data[-32:]
    return b"\x00" * (32 - len(data)) + data


class YuClient:
    """SDK client for interacting with a yu blockchain node."""

    def __init__(self, http_url: str = "http://localhost:7999", ws_url: str = "ws://localhost:8999"):
        self.http_url = http_url.rstrip("/")
        self.ws_url = ws_url.rstrip("/")
        self._keypair: Optional[KeyPair] = None

    def with_keypair(self, keypair: KeyPair) -> "YuClient":
        self._keypair = keypair
        return self

    def write_chain(self, tripod_name: str, func_name: str, params: Any, lei_price: int = 0, tips: int = 0) -> None:
        """Send a signed writing transaction to the chain."""
        if self._keypair is None:
            raise RuntimeError("KeyPair not set; call with_keypair() first")

        params_str = json.dumps(params, separators=(",", ":"))
        wr_call = WrCall(tripod_name=tripod_name, func_name=func_name, params=params_str,
                         lei_price=lei_price, tips=tips)

        # Sign: BytesToHash(json.marshal(wr_call))
        call_json = json.dumps(wr_call.to_dict(), separators=(",", ":")).encode()
        msg_hash = _bytes_to_hash(call_json)
        sig = self._keypair.sign(msg_hash)

        post_body = {
            "pubkey": self._keypair.pubkey_with_type,
            "address": self._keypair.address,
            "signature": _to_hex(sig),
            "call": wr_call.to_dict(),
        }
        resp = requests.post(f"{self.http_url}/api/writing", json=post_body)
        resp.raise_for_status()

    def read_chain(self, tripod_name: str, func_name: str, params: Any) -> Any:
        """Send a reading query to the chain. Returns the parsed JSON response body."""
        params_str = json.dumps(params, separators=(",", ":"))
        rd_call = RdCall(tripod_name=tripod_name, func_name=func_name, params=params_str)
        resp = requests.post(f"{self.http_url}/api/reading", json=rd_call.to_dict())
        resp.raise_for_status()
        return resp.json()

    def subscribe_events(self, callback: Callable[[Receipt], None]) -> "EventSubscriber":
        """
        Subscribe to chain events over WebSocket.

        Returns an EventSubscriber; call its close() method to stop.
        callback is called for each received Receipt.
        """
        sub = EventSubscriber(f"{self.ws_url}/subscribe/results", callback)
        sub.start()
        return sub

    def stop_chain(self) -> None:
        """Send admin stop request."""
        requests.get(f"{self.http_url}/api/admin/stop")


class EventSubscriber:
    """WebSocket subscriber that calls callback for each Receipt."""

    def __init__(self, url: str, callback: Callable[[Receipt], None]):
        self._url = url
        self._callback = callback
        self._ws: Optional[websocket.WebSocketApp] = None
        self._thread: Optional[threading.Thread] = None

    def start(self) -> None:
        def on_message(ws, message):
            try:
                data = json.loads(message)
                receipt = Receipt(
                    tx_hash=data.get("tx_hash", ""),
                    block_hash=data.get("block_hash", ""),
                    height=data.get("height", 0),
                    tripod_name=data.get("tripod_name", ""),
                    writing_name=data.get("writing_name", ""),
                    lei_cost=data.get("lei_cost", 0),
                    error=data.get("error", ""),
                )
                self._callback(receipt)
            except Exception:
                pass

        self._ws = websocket.WebSocketApp(self._url, on_message=on_message)
        self._thread = threading.Thread(target=self._ws.run_forever, daemon=True)
        self._thread.start()

    def close(self) -> None:
        if self._ws:
            self._ws.close()
