/**
 * YuClient: async HTTP + WebSocket client for the yu blockchain.
 *
 * Writing endpoint: POST <http_url>/api/writing
 * Reading endpoint: POST <http_url>/api/reading
 * Event stream:     WS   <ws_url>/subscribe/results
 */

use std::sync::Arc;

use futures_util::StreamExt;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;

use crate::keypair::KeyPair;
use crate::types::{RdCall, Receipt, WritingPostBody, WrCall};

/// Replicates yu common.BytesToHash:
/// takes the last 32 bytes (or zero-pads to 32 if shorter).
fn bytes_to_hash(data: &[u8]) -> [u8; 32] {
    let mut hash = [0u8; 32];
    if data.len() >= 32 {
        hash.copy_from_slice(&data[data.len() - 32..]);
    } else {
        hash[32 - data.len()..].copy_from_slice(data);
    }
    hash
}

fn to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

pub struct YuClient {
    http_url: String,
    ws_url: String,
    keypair: Option<Arc<KeyPair>>,
    http: Client,
}

impl YuClient {
    pub fn new(http_url: &str, ws_url: &str) -> Self {
        Self {
            http_url: http_url.trim_end_matches('/').to_string(),
            ws_url: ws_url.trim_end_matches('/').to_string(),
            keypair: None,
            http: Client::new(),
        }
    }

    pub fn with_keypair(mut self, keypair: KeyPair) -> Self {
        self.keypair = Some(Arc::new(keypair));
        self
    }

    /// Send a signed writing transaction to the chain.
    pub async fn write_chain(
        &self,
        tripod_name: &str,
        func_name: &str,
        params: &impl Serialize,
        lei_price: u64,
        tips: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let keypair = self.keypair.as_ref().ok_or("KeyPair not set")?;

        let params_str = serde_json::to_string(params)?;
        let wr_call = WrCall {
            tripod_name: tripod_name.to_string(),
            func_name: func_name.to_string(),
            params: params_str,
            chain_id: None,
            lei_price: if lei_price > 0 { Some(lei_price) } else { None },
            tips: if tips > 0 { Some(tips) } else { None },
        };

        let call_json = serde_json::to_vec(&wr_call)?;
        let msg_hash = bytes_to_hash(&call_json);
        let sig = keypair.sign(&msg_hash);

        let post_body = WritingPostBody {
            pubkey: keypair.pubkey_with_type(),
            address: keypair.address(),
            signature: to_hex(&sig),
            call: &wr_call,
        };

        self.http
            .post(format!("{}/api/writing", self.http_url))
            .json(&post_body)
            .send()
            .await?;

        Ok(())
    }

    /// Send a reading query to the chain. Returns the parsed JSON value.
    pub async fn read_chain(
        &self,
        tripod_name: &str,
        func_name: &str,
        params: &impl Serialize,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let params_str = serde_json::to_string(params)?;
        let rd_call = RdCall {
            tripod_name: tripod_name.to_string(),
            func_name: func_name.to_string(),
            params: params_str,
            block_hash: String::new(),
        };

        let resp = self
            .http
            .post(format!("{}/api/reading", self.http_url))
            .json(&rd_call)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(resp)
    }

    /// Subscribe to chain events. Returns a receiver channel and a JoinHandle.
    /// The handle runs until the WebSocket closes.
    pub async fn subscribe_events(
        &self,
    ) -> Result<
        (mpsc::Receiver<Receipt>, tokio::task::JoinHandle<()>),
        Box<dyn std::error::Error>,
    > {
        let url = format!("{}/subscribe/results", self.ws_url);
        let (ws_stream, _) = connect_async(url.as_str()).await?;
        let (tx, rx) = mpsc::channel(64);

        let handle = tokio::spawn(async move {
            let (_, mut read) = ws_stream.split();
            while let Some(Ok(msg)) = read.next().await {
                if let Ok(text) = msg.into_text() {
                    if let Ok(receipt) = serde_json::from_str::<Receipt>(&text) {
                        let _ = tx.send(receipt).await;
                    }
                }
            }
        });

        Ok((rx, handle))
    }

    /// Send admin stop request.
    pub async fn stop_chain(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self
            .http
            .get(format!("{}/api/admin/stop", self.http_url))
            .send()
            .await;
        Ok(())
    }
}
