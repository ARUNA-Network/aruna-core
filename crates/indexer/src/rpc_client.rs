//! Typed HTTP client for the ARUNA Node RPC.
//!
//! All methods perform a single GET request and deserialize the JSON response.
//! The indexer calls these methods; it never touches RocksDB directly.

use serde::Deserialize;
use reqwest::Client;

// ── Node RPC response shapes ──────────────────────────────────────────────────

/// `/status` response.
#[derive(Debug, Deserialize)]
pub struct NodeStatus {
    pub height: u64,
    pub best_block: String,
    pub synced: bool,
    pub peer_count: usize,
}

/// `/block/height/:n` and `/block/hash/:hash` response.
#[derive(Debug, Deserialize)]
pub struct RpcBlock {
    pub height: u64,
    pub hash: String,
    pub header: RpcBlockHeader,
    pub body: RpcBlockBody,
}

#[derive(Debug, Deserialize)]
pub struct RpcBlockHeader {
    pub version: u32,
    pub prev_block_hash: String,
    pub merkle_root: String,
    pub state_root: String,
    pub timestamp: u64,
    /// Difficulty comes as a JSON number from the node's Serialize impl.
    pub difficulty: serde_json::Value,
    pub nonce: u64,
}

impl RpcBlockHeader {
    /// Extract difficulty as u64 from whatever JSON shape the node sends.
    pub fn difficulty_u64(&self) -> u64 {
        match &self.difficulty {
            serde_json::Value::Number(n) => n.as_u64().unwrap_or(0),
            serde_json::Value::Object(m) => {
                // Struct shape: {"0": N}
                m.get("0").and_then(|v| v.as_u64()).unwrap_or(0)
            }
            _ => 0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RpcBlockBody {
    pub transactions: Vec<RpcTransaction>,
}

#[derive(Debug, Deserialize)]
pub struct RpcTransaction {
    pub payload: RpcTxPayload,
    pub signature_type: serde_json::Value,  // u8 or enum string
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl RpcTransaction {
    /// Returns signature type as u8 (0 = Ed25519, 1 = secp256k1).
    pub fn sig_type_u8(&self) -> u8 {
        match &self.signature_type {
            serde_json::Value::Number(n) => n.as_u64().unwrap_or(0) as u8,
            serde_json::Value::String(s) => match s.as_str() {
                "Ed25519"  | "ed25519"   => 0,
                "Secp256k1"| "secp256k1" => 1,
                _                        => 0,
            },
            _ => 0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RpcTxPayload {
    /// Nonce as a raw JSON value — the node may emit it as a number or as {"0": N}.
    pub nonce: serde_json::Value,
    /// Sender as a raw JSON value — may be a hex string or a bytes array.
    pub sender: serde_json::Value,
    /// Recipient as a raw JSON value.
    pub recipient: serde_json::Value,
    pub amount: u64,
    pub fee: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub data: Vec<u8>,
}

impl RpcTxPayload {
    /// Extract nonce as u64.
    pub fn nonce_u64(&self) -> u64 {
        match &self.nonce {
            serde_json::Value::Number(n) => n.as_u64().unwrap_or(0),
            serde_json::Value::Object(m) => m.get("0").and_then(|v| v.as_u64()).unwrap_or(0),
            _ => 0,
        }
    }

    /// Convert sender to a hex string representation.
    pub fn sender_hex(&self) -> String {
        json_value_to_hex(&self.sender)
    }

    /// Convert recipient to a hex string representation.
    pub fn recipient_hex(&self) -> String {
        json_value_to_hex(&self.recipient)
    }
}

/// Convert a JSON value (string or bytes array) to a hex string.
fn json_value_to_hex(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            // Array of u8 numbers
            let bytes: Vec<u8> = arr.iter()
                .filter_map(|b| b.as_u64().map(|n| n as u8))
                .collect();
            hex::encode(bytes)
        }
        serde_json::Value::Object(m) => {
            // {"0": [u8; 32]} struct format
            if let Some(arr) = m.get("0").and_then(|v| v.as_array()) {
                let bytes: Vec<u8> = arr.iter()
                    .filter_map(|b| b.as_u64().map(|n| n as u8))
                    .collect();
                hex::encode(bytes)
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

/// `/account/:address` response.
#[derive(Debug, Deserialize)]
pub struct RpcAccount {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
}

// ── Client ────────────────────────────────────────────────────────────────────

/// Typed HTTP client wrapping `reqwest::Client` for the Node RPC.
#[derive(Clone)]
pub struct RpcClient {
    client: Client,
    base_url: String,
}

impl RpcClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to build HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Fetch node status (height, best_block, synced).
    pub async fn get_status(&self) -> Result<NodeStatus, reqwest::Error> {
        self.client
            .get(format!("{}/status", self.base_url))
            .send()
            .await?
            .json::<NodeStatus>()
            .await
    }

    /// Fetch a block by height.
    pub async fn get_block_by_height(&self, height: u64) -> Result<RpcBlock, reqwest::Error> {
        self.client
            .get(format!("{}/block/height/{}", self.base_url, height))
            .send()
            .await?
            .json::<RpcBlock>()
            .await
    }

    /// Fetch account state by address string.
    pub async fn get_account(&self, address: &str) -> Result<RpcAccount, reqwest::Error> {
        self.client
            .get(format!("{}/account/{}", self.base_url, address))
            .send()
            .await?
            .json::<RpcAccount>()
            .await
    }
}
