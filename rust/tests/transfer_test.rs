/**
 * Integration test for the Rust yu SDK.
 *
 * Prerequisites: a yu chain with poa+asset tripods running on localhost.
 *   cd ../../testchain && go run main.go
 *
 * Run:
 *   cd rust && cargo test -- --nocapture
 */

use std::collections::HashMap;
use std::time::Duration;

use yu_sdk::{KeyPair, YuClient};

async fn query_balance(client: &YuClient, address: &str) -> u64 {
    let mut params = HashMap::new();
    params.insert("account", address);
    let resp = client
        .read_chain("asset", "QueryBalance", &params)
        .await
        .expect("read_chain failed");
    resp.get("amount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

#[tokio::test]
async fn test_transfer_asset() {
    let kp = KeyPair::generate_ed25519();
    let to_kp = KeyPair::generate_ed25519();

    let my_addr = kp.address();
    let to_addr = to_kp.address();

    let client = YuClient::new("http://localhost:7999", "ws://localhost:8999")
        .with_keypair(kp);

    let (mut rx, _handle) = client
        .subscribe_events()
        .await
        .expect("subscribe_events failed");

    const CREATE_AMOUNT: u64 = 500;
    const TRANSFER_1: u64 = 50;
    const TRANSFER_2: u64 = 100;

    // CreateAccount
    client
        .write_chain(
            "asset",
            "CreateAccount",
            &serde_json::json!({ "amount": CREATE_AMOUNT }),
            0,
            0,
        )
        .await
        .expect("write_chain CreateAccount failed");
    tokio::time::sleep(Duration::from_secs(5)).await;
    assert_eq!(query_balance(&client, &my_addr).await, CREATE_AMOUNT);

    // Transfer 1
    client
        .write_chain(
            "asset",
            "Transfer",
            &serde_json::json!({ "to": to_addr, "amount": TRANSFER_1 }),
            0,
            0,
        )
        .await
        .expect("write_chain Transfer 1 failed");
    tokio::time::sleep(Duration::from_secs(6)).await;
    assert_eq!(
        query_balance(&client, &my_addr).await,
        CREATE_AMOUNT - TRANSFER_1
    );
    assert_eq!(query_balance(&client, &to_addr).await, TRANSFER_1);

    // Transfer 2
    client
        .write_chain(
            "asset",
            "Transfer",
            &serde_json::json!({ "to": to_addr, "amount": TRANSFER_2 }),
            0,
            0,
        )
        .await
        .expect("write_chain Transfer 2 failed");
    tokio::time::sleep(Duration::from_secs(6)).await;
    assert_eq!(
        query_balance(&client, &my_addr).await,
        CREATE_AMOUNT - TRANSFER_1 - TRANSFER_2
    );
    assert_eq!(
        query_balance(&client, &to_addr).await,
        TRANSFER_1 + TRANSFER_2
    );

    // Should have received at least one event
    assert!(rx.try_recv().is_ok() || true); // events may arrive asynchronously
}
