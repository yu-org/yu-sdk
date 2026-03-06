# yu-sdk/rust

Rust SDK for interacting with a [yu](https://github.com/yu-org/yu) blockchain node.

## Features

- **Writing**: send signed transactions to the chain (`/api/writing`)
- **Reading**: query chain state (`/api/reading`)
- **Event subscription**: receive receipts via WebSocket (`/subscribe/results`)
- **Key types**: Ed25519 (via `ed25519-dalek`)

## Usage

```rust
use yu_sdk::{YuClient, KeyPair};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let kp = KeyPair::generate_ed25519();
    let my_addr = kp.address();

    let client = YuClient::new("http://localhost:7999", "ws://localhost:8999")
        .with_keypair(kp);

    // Writing call
    client.write_chain(
        "asset", "CreateAccount",
        &serde_json::json!({ "amount": 500u64 }),
        0, 0,
    ).await.unwrap();

    // Reading call
    let mut params = HashMap::new();
    params.insert("account", my_addr.as_str());
    let resp = client.read_chain("asset", "QueryBalance", &params).await.unwrap();
    println!("balance: {}", resp["amount"]);

    // Subscribe to events
    let (mut rx, _handle) = client.subscribe_events().await.unwrap();
    while let Some(receipt) = rx.recv().await {
        println!("receipt: {:?}", receipt);
    }
}
```

## Signing

Writing transactions are signed using:

1. JSON-serialize the `WrCall` struct
2. Take the last 32 bytes (`BytesToHash`)
3. Sign with Ed25519 private key using `ed25519-dalek`

## Running Tests

Start the test chain first:

```bash
cd ../testchain
go mod tidy
go run main.go
```

Then in a separate terminal:

```bash
cd rust
cargo test -- --nocapture
```
