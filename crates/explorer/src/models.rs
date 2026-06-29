//! API response models (JSON-serializable types) for the Explorer REST API.

use serde::Serialize;

/// Single block summary (used in paginated lists).
#[derive(Debug, Serialize)]
pub struct BlockSummary {
    pub height: i64,
    pub hash: String,
    pub prev_hash: String,
    pub merkle_root: String,
    pub state_root: String,
    pub timestamp: i64,
    pub difficulty: i64,
    pub nonce: i64,
    pub version: i32,
    pub tx_count: i32,
}

/// Detailed block response (includes transaction list).
#[derive(Debug, Serialize)]
pub struct BlockDetail {
    pub height: i64,
    pub hash: String,
    pub prev_hash: String,
    pub merkle_root: String,
    pub state_root: String,
    pub timestamp: i64,
    pub difficulty: i64,
    pub nonce: i64,
    pub version: i32,
    pub tx_count: i32,
    pub transactions: Vec<TxSummary>,
}

/// Transaction summary (used in block detail & address tx history).
#[derive(Debug, Serialize)]
pub struct TxSummary {
    pub hash: String,
    pub block_height: i64,
    pub block_hash: String,
    pub tx_index: i32,
    pub sender: String,
    pub recipient: String,
    pub amount: i64,
    pub fee: i64,
    pub nonce_val: i64,
    pub gas_limit: i64,
    pub gas_price: i64,
    pub has_data: bool,
    pub sig_type: i16,
}

/// Account/address state response.
#[derive(Debug, Serialize)]
pub struct AddressDetail {
    pub address: String,
    pub balance: i64,
    pub nonce: i64,
    pub updated_at_block: i64,
    pub transactions: Vec<TxSummary>,
}

/// Global chain statistics.
#[derive(Debug, Serialize)]
pub struct ChainStats {
    pub height: i64,
    pub total_tx_count: i64,
    pub best_hash: String,
    pub last_block_time: i64,
}

/// Paginated response wrapper.
#[derive(Debug, Serialize)]
pub struct Paginated<T: Serialize> {
    pub items: Vec<T>,
    pub total: Option<i64>,
    pub limit: i64,
    pub offset: i64,
}

/// Search result item.
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub kind: String,  // "block", "transaction", "address"
    pub value: String, // hash / height / address
}

/// Generic error response.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Generic success response.
#[derive(Debug, Serialize)]
pub struct OkResponse {
    pub status: String,
}
