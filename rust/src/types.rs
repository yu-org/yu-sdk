use serde::{Deserialize, Serialize};

/// A writing call from a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrCall {
    pub tripod_name: String,
    pub func_name: String,
    pub params: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lei_price: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tips: Option<u64>,
}

/// A reading call from a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdCall {
    pub tripod_name: String,
    pub func_name: String,
    pub params: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub block_hash: String,
}

impl Default for RdCall {
    fn default() -> Self {
        Self {
            tripod_name: String::new(),
            func_name: String::new(),
            params: String::new(),
            block_hash: String::new(),
        }
    }
}

/// The POST body for a writing request.
#[derive(Debug, Serialize)]
pub struct WritingPostBody<'a> {
    pub pubkey: String,
    pub address: String,
    pub signature: String,
    pub call: &'a WrCall,
}

/// A transaction receipt received from the WebSocket event stream.
#[derive(Debug, Clone, Deserialize)]
pub struct Receipt {
    pub tx_hash: Option<String>,
    pub block_hash: Option<String>,
    pub height: Option<u64>,
    pub tripod_name: Option<String>,
    pub writing_name: Option<String>,
    pub lei_cost: Option<u64>,
    pub error: Option<String>,
}
