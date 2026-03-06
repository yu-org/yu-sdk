# yu-sdk/python

Python SDK for interacting with a [yu](https://github.com/yu-org/yu) blockchain node.

## Features

- **Writing**: send signed transactions to the chain (`/api/writing`)
- **Reading**: query chain state (`/api/reading`)
- **Event subscription**: receive receipts via WebSocket (`/subscribe/results`)
- **Key types**: Ed25519 (built-in), Sr25519 (`py-sr25519-bindings`), Secp256k1 (`eth-keys`)

## Installation

```bash
pip install -r requirements.txt
```

## Usage

```python
from yu_sdk import YuClient, KeyPair, KeyType

# Generate key pair (ed25519 by default)
kp = KeyPair(KeyType.ED25519)

# Create client
client = YuClient("http://localhost:7999", "ws://localhost:8999")
client.with_keypair(kp)

# Writing call
client.write_chain("asset", "CreateAccount", {"amount": 500})

# Reading call
resp = client.read_chain("asset", "QueryBalance", {"account": kp.address})
balance = resp["amount"]

# Subscribe to events
def on_event(receipt):
    print("Received:", receipt)

sub = client.subscribe_events(on_event)
# ... do work ...
sub.close()
```

## Signing

Writing transactions are signed using the following algorithm (matching yu's Go implementation):

1. JSON-serialize the `WrCall` struct
2. Take the last 32 bytes of the serialized JSON (`BytesToHash`)
3. Sign those 32 bytes with the private key

The public key is sent as `0x<type_byte><pubkey_hex>` where type bytes are:
- Sr25519: `31` (ASCII `'1'`)
- Ed25519: `32` (ASCII `'2'`)
- Secp256k1: `31` (matches yu source)

## Running Tests

Start the test chain first (requires Go and the yu repository):

```bash
cd ../testchain
go mod tidy
go run main.go
```

Then in a separate terminal:

```bash
cd python
pip install -r requirements.txt
pytest tests/ -v
```
