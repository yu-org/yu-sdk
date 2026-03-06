# yu-sdk

Multi-language SDK for interacting with a [yu](https://github.com/yu-org/yu) blockchain node.

## Supported Languages

| Language   | Directory  | Key Types                   |
|------------|------------|-----------------------------|
| Go         | `go/`      | Sr25519, Ed25519, Secp256k1 |
| Python     | `python/`  | Ed25519 (built-in), Sr25519*, Secp256k1* |
| JavaScript | `js/`      | Ed25519 (built-in), Sr25519* |
| Rust       | `rust/`    | Ed25519 (built-in)          |

\* requires optional dependency

## Features

All SDKs support:

- **Writing** — send a signed transaction: `POST /api/writing`
- **Reading** — query chain state: `POST /api/reading`
- **Event subscription** — receive receipts via WebSocket: `ws://.../subscribe/results`

## Quick Start

### 1. Start the test chain

```bash
cd testchain
go mod tidy
go run main.go
```

This starts a single-node yu chain with **poa** and **asset** tripods on:
- HTTP: `localhost:7999`
- WebSocket: `localhost:8999`

### 2. Run SDK tests

**Go**
```bash
cd go && go mod tidy && go test ./tests/ -v -timeout 120s
```

**Python**
```bash
cd python && pip install -r requirements.txt && pytest tests/ -v
```

**JavaScript**
```bash
cd js && npm install && npm test
```

**Rust**
```bash
cd rust && cargo test -- --nocapture
```

## Directory Structure

```
yu-sdk/
├── testchain/        # Go program to start a poa+asset chain for testing
├── go/               # Go SDK
├── python/           # Python SDK
├── js/               # JavaScript SDK
└── rust/             # Rust SDK
```

## Signing Algorithm

Writing transactions are signed using yu's convention:

1. JSON-serialize the `WrCall` struct
2. `msg = last_32_bytes(json_bytes)` — equivalent to `BytesToHash` in yu
3. `signature = sign(private_key, msg)`

Public key format: `"0x" + hex(type_prefix_byte + pubkey_bytes)`
- Sr25519: prefix `0x31` (ASCII `'1'`)
- Ed25519: prefix `0x32` (ASCII `'2'`)