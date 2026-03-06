# yu-sdk/js

JavaScript SDK for interacting with a [yu](https://github.com/yu-org/yu) blockchain node.

## Features

- **Writing**: send signed transactions to the chain (`/api/writing`)
- **Reading**: query chain state (`/api/reading`)
- **Event subscription**: receive receipts via WebSocket (`/subscribe/results`)
- **Key types**: Ed25519 (built-in via `@noble/ed25519`), Sr25519 (optional via `@polkadot/util-crypto`)

## Installation

```bash
npm install
```

## Usage

```js
import { YuClient, KeyPair } from "@yu-org/yu-sdk";

// Generate Ed25519 key pair
const kp = new KeyPair();

// Create client
const client = new YuClient("http://localhost:7999", "ws://localhost:8999");
client.withKeypair(kp);

// Writing call
await client.writeChain("asset", "CreateAccount", { amount: 500 });

// Reading call
const resp = await client.readChain("asset", "QueryBalance", { account: await kp.getAddress() });
console.log("balance:", resp.amount);

// Subscribe to events
const sub = client.subscribeEvents((receipt) => {
  console.log("event:", receipt);
});
// ... later ...
sub.close();
```

## Signing

Writing transactions are signed using:

1. JSON-serialize the `WrCall` object
2. Take the last 32 bytes (`BytesToHash`)
3. Sign with Ed25519 private key

## Running Tests

Start the test chain first:

```bash
cd ../testchain
go mod tidy
go run main.go
```

Then in a separate terminal:

```bash
cd js
npm install
npm test
```
