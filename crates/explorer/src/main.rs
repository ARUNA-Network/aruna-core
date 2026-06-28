//! ARUNA Network Explorer API Backend
//!
//! A read-only REST API that serves chain data from a RocksDB storage instance.
//! Designed to run as a sidecar alongside an ARUNA node, pointing at the same
//! RocksDB directory via a secondary read-only connection.
//!
//! # Endpoints
//! - GET /block/latest           — best block header + metadata
//! - GET /block/height/<n>       — block header + tx hashes at height n
//! - GET /block/hash/<hash>      — block by hash hex
//! - GET /address/<addr>         — account balance + nonce
//! - GET /transaction/<hash>     — tx status + block height + confirmations
//! - GET /stats                  — chain stats (height, best hash, etc.)

use std::sync::Arc;
use axum::{extract::{Path, State}, response::Json, routing::get, Router};
use serde_json::{json, Value};
use aruna_storage::Storage;
use aruna_primitives::{Address, Hash};

/// Shared application state: a read-only storage handle.
#[derive(Clone)]
struct AppState {
    storage: Arc<Storage>,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let db_path = args.iter()
        .position(|a| a == "--db")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "./aruna-data".to_string());
    let listen = args.iter()
        .position(|a| a == "--listen")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "127.0.0.1:9090".to_string());

    println!("ARUNA Explorer API starting...");
    println!("DB path: {}", db_path);
    println!("Listening on: http://{}", listen);

    let storage = Storage::open(std::path::Path::new(&db_path))
        .expect("Failed to open RocksDB. Ensure the node has initialized the database first.");
    let state = AppState { storage: Arc::new(storage) };

    let app = Router::new()
        .route("/block/latest", get(block_latest))
        .route("/block/height/{n}", get(block_by_height))
        .route("/block/hash/{hash}", get(block_by_hash))
        .route("/address/{addr}", get(address_info))
        .route("/transaction/{hash}", get(transaction_info))
        .route("/stats", get(chain_stats))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen).await
        .expect("Failed to bind listener");
    axum::serve(listener, app).await.expect("Server failed");
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn block_latest(State(state): State<AppState>) -> Json<Value> {
    let storage = &state.storage;
    match storage.get_best_block() {
        Ok(Some(best_hash)) => {
            match storage.get_block_header(&best_hash) {
                Ok(Some(header)) => {
                    let height = storage.get_chain_height()
                        .ok().flatten().unwrap_or(0);
                    Json(json!({
                        "height": height,
                        "hash": hex::encode(best_hash.0),
                        "timestamp": header.timestamp,
                        "difficulty": header.difficulty.0,
                        "state_root": hex::encode(header.state_root.0),
                        "prev_hash": hex::encode(header.prev_block_hash.0),
                        "merkle_root": hex::encode(header.merkle_root.0),
                    }))
                }
                _ => Json(json!({"error": "Block header not found"})),
            }
        }
        _ => Json(json!({"error": "No best block set — chain may not be initialized"})),
    }
}

async fn block_by_height(
    State(state): State<AppState>,
    Path(n): Path<u64>,
) -> Json<Value> {
    let storage = &state.storage;
    match storage.get_block_hash_by_height(n) {
        Ok(Some(hash)) => match storage.get_block_header(&hash) {
            Ok(Some(header)) => {
                let tx_hashes = storage.get_block_body(&hash)
                    .ok().flatten()
                    .map(|body| body.transactions.iter()
                        .map(|tx| {
                            let bytes = bincode::serialize(tx).unwrap_or_default();
                            hex::encode(aruna_crypto::blake3_hash(&bytes).0)
                        })
                        .collect::<Vec<_>>())
                    .unwrap_or_default();
                Json(json!({
                    "height": n,
                    "hash": hex::encode(hash.0),
                    "timestamp": header.timestamp,
                    "difficulty": header.difficulty.0,
                    "state_root": hex::encode(header.state_root.0),
                    "tx_count": tx_hashes.len(),
                    "tx_hashes": tx_hashes,
                }))
            }
            _ => Json(json!({"error": "Block header not found"})),
        },
        _ => Json(json!({"error": format!("No block at height {}", n)})),
    }
}

async fn block_by_hash(
    State(state): State<AppState>,
    Path(hash_hex): Path<String>,
) -> Json<Value> {
    let storage = &state.storage;
    let hash_bytes: [u8; 32] = match hex::decode(&hash_hex)
        .ok()
        .and_then(|b| b.try_into().ok())
    {
        Some(b) => b,
        None => return Json(json!({"error": "Invalid hash hex (must be 64 hex chars)"})),
    };
    let hash = Hash(hash_bytes);
    match storage.get_block_header(&hash) {
        Ok(Some(header)) => {
            let height = storage.get_block_height_by_hash(&hash)
                .ok().flatten().unwrap_or(0);
            let tx_hashes = storage.get_block_body(&hash)
                .ok().flatten()
                .map(|body| body.transactions.iter()
                    .map(|tx| {
                        let bytes = bincode::serialize(tx).unwrap_or_default();
                        hex::encode(aruna_crypto::blake3_hash(&bytes).0)
                    })
                    .collect::<Vec<_>>())
                .unwrap_or_default();
            Json(json!({
                "height": height,
                "hash": hash_hex,
                "timestamp": header.timestamp,
                "difficulty": header.difficulty.0,
                "state_root": hex::encode(header.state_root.0),
                "prev_hash": hex::encode(header.prev_block_hash.0),
                "tx_count": tx_hashes.len(),
                "tx_hashes": tx_hashes,
            }))
        }
        _ => Json(json!({"error": "Block not found"})),
    }
}

async fn address_info(
    State(state): State<AppState>,
    Path(addr_str): Path<String>,
) -> Json<Value> {
    let storage = &state.storage;
    let address = match Address::from_bech32m(&addr_str) {
        Ok((_, addr)) => addr,
        Err(_) => {
            // Try hex fallback
            match hex::decode(&addr_str)
                .ok()
                .and_then(|b| b.try_into().ok())
                .map(Address::new)
            {
                Some(addr) => addr,
                None => return Json(json!({"error": "Invalid address format"})),
            }
        }
    };
    match storage.get_account(&address) {
        Ok(Some((balance, nonce, _, _))) => Json(json!({
            "address": addr_str,
            "balance": balance,
            "nonce": nonce,
        })),
        Ok(None) => Json(json!({
            "address": addr_str,
            "balance": 0,
            "nonce": 0,
        })),
        Err(e) => Json(json!({"error": format!("Storage error: {:?}", e)})),
    }
}

async fn transaction_info(
    State(state): State<AppState>,
    Path(hash_hex): Path<String>,
) -> Json<Value> {
    let storage = &state.storage;
    let hash_bytes: [u8; 32] = match hex::decode(&hash_hex)
        .ok()
        .and_then(|b| b.try_into().ok())
    {
        Some(b) => b,
        None => return Json(json!({"error": "Invalid tx hash hex"})),
    };
    let hash = Hash(hash_bytes);

    // Look up the tx index: which block contains this tx
    match storage.get_tx_index(&hash) {
        Ok(Some((block_hash, _tx_index))) => {
            let block_height = storage.get_block_height_by_hash(&block_hash)
                .ok().flatten().unwrap_or(0);
            let best_height = storage.get_chain_height()
                .ok().flatten().unwrap_or(0);
            let confirmations = best_height.saturating_sub(block_height) + 1;
            Json(json!({
                "tx_hash": hash_hex,
                "status": "confirmed",
                "block_hash": hex::encode(block_hash.0),
                "block_height": block_height,
                "confirmations": confirmations,
            }))
        }
        Ok(None) => Json(json!({
            "tx_hash": hash_hex,
            "status": "not_found",
        })),
        Err(e) => Json(json!({"error": format!("Storage error: {:?}", e)})),
    }
}

async fn chain_stats(State(state): State<AppState>) -> Json<Value> {
    let storage = &state.storage;
    let height = storage.get_chain_height().ok().flatten().unwrap_or(0);
    let best_hash = storage.get_best_block().ok().flatten()
        .map(|h| hex::encode(h.0))
        .unwrap_or_else(|| "none".to_string());
    Json(json!({
        "chain_height": height,
        "best_hash": best_hash,
    }))
}
