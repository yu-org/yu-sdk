"""
Integration test for the Python yu SDK.

Prerequisites:
  - A yu chain with poa+asset tripods must be running on localhost:7999 (HTTP)
    and localhost:8999 (WebSocket).
  - Start the chain:
      cd ../../testchain && go run main.go

Run tests:
  cd python
  pip install -r requirements.txt
  pytest tests/ -v
"""

import time

import pytest

from yu_sdk import KeyPair, KeyType, YuClient
from yu_sdk.types import Receipt


@pytest.fixture(scope="module")
def client():
    kp = KeyPair(KeyType.ED25519)
    c = YuClient("http://localhost:7999", "ws://localhost:8999")
    c.with_keypair(kp)
    return c, kp


def test_create_and_transfer(client):
    c, kp = client

    to_kp = KeyPair(KeyType.ED25519)

    received: list[Receipt] = []
    sub = c.subscribe_events(lambda r: received.append(r))

    CREATE_AMOUNT = 500
    TRANSFER_1 = 50
    TRANSFER_2 = 100

    # CreateAccount
    c.write_chain("asset", "CreateAccount", {"amount": CREATE_AMOUNT})
    time.sleep(8)

    balance = _query_balance(c, kp.address)
    assert balance == CREATE_AMOUNT, f"expected {CREATE_AMOUNT}, got {balance}"

    # Transfer 1
    c.write_chain("asset", "Transfer", {"to": to_kp.address, "amount": TRANSFER_1})
    time.sleep(6)

    assert _query_balance(c, kp.address) == CREATE_AMOUNT - TRANSFER_1
    assert _query_balance(c, to_kp.address) == TRANSFER_1

    # Transfer 2
    c.write_chain("asset", "Transfer", {"to": to_kp.address, "amount": TRANSFER_2})
    time.sleep(6)

    assert _query_balance(c, kp.address) == CREATE_AMOUNT - TRANSFER_1 - TRANSFER_2
    assert _query_balance(c, to_kp.address) == TRANSFER_1 + TRANSFER_2

    sub.close()
    assert len(received) >= 1, "should have received at least one event"


def _query_balance(client: YuClient, address: str) -> int:
    resp = client.read_chain("asset", "QueryBalance", {"account": address})
    return int(resp.get("amount", 0))
