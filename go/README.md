# yu-sdk/go

Go SDK for interacting with a [yu](https://github.com/yu-org/yu) blockchain node.

## Features

- **Writing**: send signed transactions to the chain (`/api/writing`)
- **Reading**: query chain state (`/api/reading`)
- **Event subscription**: receive receipts via WebSocket (`/subscribe/results`)

## Installation

```bash
go get github.com/yu-org/yu-sdk/go
```

## Usage

```go
import (
    yusdk "github.com/yu-org/yu-sdk/go"
    "github.com/yu-org/yu/core/keypair"
)

// Create client
client := yusdk.NewClient("http://localhost:7999")

// Generate key pair
pubkey, privkey, _ := keypair.GenKeyPair(keypair.Sr25519)
client.WithKeys(privkey, pubkey)

// Writing call
err := client.WriteChain("asset", "CreateAccount", map[string]any{"amount": uint64(500)}, 0, 0)

// Reading call
resp, err := client.ReadChain("asset", "QueryBalance", map[string]string{"account": addr})

// Subscribe to events
sub, _ := client.NewSubscriber()
ch := make(chan *types.Receipt)
go sub.SubEvent(ch)
// Read from ch...
sub.Close()
```

## Running Tests

First start the test chain (in one terminal):

```bash
cd ../testchain
go mod tidy
go run main.go
```

Or run the test which starts the chain automatically:

```bash
cd go
go mod tidy
go test ./tests/ -v -timeout 120s
```
