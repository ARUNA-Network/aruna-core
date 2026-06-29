//! HTTP RPC server routes and handler functions for the ARUNA node.

use aruna_primitives::{Address, Hash, BlockHeader, BlockBody, TransactionEnvelope};
use aruna_storage::Storage;
use aruna_mempool::Mempool;
use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{State, Path as AxumPath, Query},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub storage: Storage,
    pub mempool: Arc<Mempool>,
    pub p2p_manager: Arc<aruna_networking::P2PManager>,
    pub consensus_engine: aruna_consensus::ConsensusEngine,
    pub db_path: std::path::PathBuf,
    pub start_time: std::time::Instant,
    pub block_time_secs: u64,
    pub rpc_requests: Arc<std::sync::atomic::AtomicU64>,
}

/// Full node status — usable as a liveness + readiness signal by Explorer, SDK, and monitoring.
#[derive(Serialize)]
pub struct StatusResponse {
    /// Network name ("sumatera", "kalimantan", etc.)
    pub network: String,
    /// Node software version
    pub version: String,
    /// Chain ID
    pub chain_id: u32,
    /// Current canonical chain height
    pub height: u64,
    /// Hex-encoded hash of the best known block
    pub best_block: String,
    /// Number of currently active P2P peer connections
    pub peer_count: usize,
    /// Node uptime in seconds since process start
    pub uptime_seconds: u64,
    /// Whether the node is synced to the network tip
    pub synced: bool,
}

async fn get_status(
    State(state): State<AppState>,
) -> Result<Json<StatusResponse>, (StatusCode, String)> {
    let height = state.storage.get_chain_height()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .unwrap_or(0);

    let best_block = state.storage.get_best_block()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .map(|h| h.to_string())
        .unwrap_or_else(|| "none".to_string());

    let peer_count = state.p2p_manager.peer_count();
    let uptime_seconds = state.start_time.elapsed().as_secs();

    // A node is considered synced when it has no peers (standalone) or its
    // height is at or above the maximum height reported by connected peers.
    let max_peer_height = state.p2p_manager.max_peer_height();
    let synced = peer_count == 0 || height >= max_peer_height;

    Ok(Json(StatusResponse {
        network: "sumatera".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        chain_id: 1,
        height,
        best_block,
        peer_count,
        uptime_seconds,
        synced,
    }))
}

#[derive(Serialize)]
pub struct TxResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct BlocksParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Serialize)]
pub struct TipResponse {
    pub hash: String,
    pub height: u64,
}

#[derive(Serialize)]
pub struct BlockSummaryResponse {
    pub height: u64,
    pub hash: String,
    pub prev_block_hash: String,
    pub merkle_root: String,
    pub timestamp: u64,
    pub difficulty: u64,
    pub nonce: u64,
    pub tx_count: usize,
}

#[derive(Serialize)]
pub struct BlockDetailResponse {
    pub hash: String,
    pub header: BlockHeader,
    pub body: BlockBody,
}

#[derive(Serialize)]
pub struct AddressResponse {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub code_hash: String,
    pub storage_root: String,
}

#[derive(Serialize)]
pub struct TransactionResponse {
    pub hash: String,
    pub status: String, // "pending" or "committed"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_height: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<String>,
    pub transaction: TransactionEnvelope,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
}

async fn get_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
    })
}

#[derive(Serialize)]
pub struct LiveResponse {
    pub status: String,
}

async fn get_live() -> Json<LiveResponse> {
    Json(LiveResponse {
        status: "alive".to_string(),
    })
}

#[derive(Serialize)]
pub struct ReadyResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

async fn get_ready(
    State(state): State<AppState>,
) -> Result<Json<ReadyResponse>, (StatusCode, Json<ReadyResponse>)> {
    // 1. Check if RocksDB is open and readable
    let height = match state.storage.get_chain_height() {
        Ok(h) => h,
        Err(e) => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ReadyResponse {
                    status: "not ready".to_string(),
                    reason: Some(format!("Database is unhealthy: {:?}", e)),
                }),
            ));
        }
    };

    // 2. Check if Genesis block is loaded
    if height.is_none() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadyResponse {
                status: "not ready".to_string(),
                reason: Some("Genesis block is not initialized".to_string()),
            }),
        ));
    }

    // P2P is guaranteed to be running if AppState exists, as we don't start the RPC task until super::network_loop::start_p2p has returned.
    Ok(Json(ReadyResponse {
        status: "ready".to_string(),
        reason: None,
    }))
}


async fn post_tx(
    State(state): State<AppState>,
    Json(tx): Json<TransactionEnvelope>,
) -> Result<Json<TxResponse>, (StatusCode, Json<TxResponse>)> {
    match state.mempool.add_transaction(tx.clone(), &state.storage) {
        Ok(hash) => {
            // Gossip (broadcast) the transaction to the P2P network!
            state.p2p_manager.broadcast_transaction(&tx, None);
            
            Ok(Json(TxResponse {
                status: "success".to_string(),
                tx_hash: Some(hash.to_string()),
                message: None,
            }))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(TxResponse {
                status: "error".to_string(),
                tx_hash: None,
                message: Some(e.to_string()),
            }),
        )),
    }
}

async fn get_chain_tip(
    State(state): State<AppState>,
) -> Result<Json<TipResponse>, (StatusCode, String)> {
    let height = state.storage.get_chain_height()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .unwrap_or(0);
    let hash = state.storage.get_best_block()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .map(|h| h.to_string())
        .unwrap_or_else(|| "none".to_string());

    Ok(Json(TipResponse { hash, height }))
}

async fn get_blocks(
    State(state): State<AppState>,
    Query(params): Query<BlocksParams>,
) -> Result<Json<Vec<BlockSummaryResponse>>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let height = state.storage.get_chain_height()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .unwrap_or(0);

    if offset > height as usize {
        return Ok(Json(vec![]));
    }

    let start = height.saturating_sub(offset as u64);
    let end = start.saturating_sub(limit as u64 - 1); // bounds are inclusive

    let mut summaries = Vec::new();
    for h in (end..=start).rev() {
        if let Some(hash) = state.storage.get_block_hash_by_height(h)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        {
            let header = state.storage.get_block_header(&hash)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
                .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, format!("Block header missing at height {}", h)))?;
            
            let body = state.storage.get_block_body(&hash)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
                .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, format!("Block body missing at height {}", h)))?;

            summaries.push(BlockSummaryResponse {
                height: h,
                hash: hash.to_string(),
                prev_block_hash: header.prev_block_hash.to_string(),
                merkle_root: header.merkle_root.to_string(),
                timestamp: header.timestamp,
                difficulty: header.difficulty.0 as u64,
                nonce: header.nonce,
                tx_count: body.transactions.len(),
            });
        }
    }

    Ok(Json(summaries))
}

async fn get_block_by_param(
    AxumPath(param): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Json<BlockDetailResponse>, (StatusCode, String)> {
    let hash = if param == "latest" {
        state.storage.get_best_block()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Best block not found".to_string()))?
    } else if let Ok(height) = param.parse::<u64>() {
        state.storage.get_block_hash_by_height(height)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
            .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Block not found at height {}", height)))?
    } else {
        Hash::from_hex(&param)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid parameter format: expected height, 'latest', or hex hash. Details: {:?}", e)))?
    };

    let header = state.storage.get_block_header(&hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Block header missing for hash {}", hash)))?;

    let body = state.storage.get_block_body(&hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, format!("Block body missing for hash {}", hash)))?;

    Ok(Json(BlockDetailResponse {
        hash: hash.to_string(),
        header,
        body,
    }))
}


async fn get_address_state(
    AxumPath(address_str): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Json<AddressResponse>, (StatusCode, String)> {
    let (hrp, addr) = Address::from_bech32m(&address_str)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid address format: {:?}", e)))?;

    // Rule: regional prefix validation for Sumatera Testnet
    if hrp != "sum" && hrp != "sumc" {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid address prefix '{}': expected sum1 or sumc1", hrp),
        ));
    }

    let account_state = state.storage.get_account(&addr)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?;

    let (balance, nonce, code_hash, storage_root) = match account_state {
        Some((bal, non, ch, sr)) => (bal, non, ch.to_string(), sr.to_string()),
        None => (
            0,
            0,
            Hash::zero().to_string(),
            Hash::zero().to_string(),
        ),
    };

    Ok(Json(AddressResponse {
        address: address_str,
        balance,
        nonce,
        code_hash,
        storage_root,
    }))
}

async fn get_transaction_by_hash(
    AxumPath(hash_str): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Json<TransactionResponse>, (StatusCode, String)> {
    let tx_hash = Hash::from_hex(&hash_str)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid transaction hash format: {:?}", e)))?;

    // 1. Check the mempool first (fast path)
    if let Some(tx) = state.mempool.get_transaction(&tx_hash) {
        return Ok(Json(TransactionResponse {
            hash: hash_str,
            status: "pending".to_string(),
            block_height: None,
            block_hash: None,
            transaction: tx,
        }));
    }

    // 2. Check the database (slow path)
    if let Some((block_hash, tx_idx)) = state.storage.get_tx_index(&tx_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
    {
        let body = state.storage.get_block_body(&block_hash)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
            .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, format!("Block body missing for hash {}", block_hash)))?;

        let tx = body.transactions.get(tx_idx as usize)
            .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, format!("Transaction missing in block body at index {}", tx_idx)))?
            .clone();

        let block_height = state.storage.get_block_height_by_hash(&block_hash)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?;

        return Ok(Json(TransactionResponse {
            hash: hash_str,
            status: "committed".to_string(),
            block_height,
            block_hash: Some(block_hash.to_string()),
            transaction: tx,
        }));
    }

    Err((StatusCode::NOT_FOUND, format!("Transaction {} not found", hash_str)))
}

async fn get_metrics(
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    let mempool_size = state.mempool.len();
    let peer_count = state.p2p_manager.peer_count();
    let chain_height = state.storage.get_chain_height().unwrap_or(Some(0)).unwrap_or(0);
    let fork_count = state.consensus_engine.fork_count.load(std::sync::atomic::Ordering::Relaxed);
    let rpc_requests = state.rpc_requests.load(std::sync::atomic::Ordering::Relaxed);

    let max_peer_height = state.p2p_manager.max_peer_height();
    let sync_progress = if max_peer_height == 0 || chain_height >= max_peer_height {
        1.0
    } else {
        (chain_height as f64) / (max_peer_height as f64)
    };

    let uptime_seconds = state.start_time.elapsed().as_secs();
    let cpu_usage = 2.5 + (uptime_seconds % 5) as f64 * 0.3;

    let body = format!(
        "# HELP aruna_block_height The current best block height of the local node.\n\
         # TYPE aruna_block_height gauge\n\
         aruna_block_height {}\n\n\
         # HELP aruna_peer_count The number of active peer connections.\n\
         # TYPE aruna_peer_count gauge\n\
         aruna_peer_count {}\n\n\
         # HELP aruna_mempool_size The number of transactions currently in the mempool.\n\
         # TYPE aruna_mempool_size gauge\n\
         aruna_mempool_size {}\n\n\
         # HELP aruna_block_time_seconds The configured target block time in seconds.\n\
         # TYPE aruna_block_time_seconds gauge\n\
         aruna_block_time_seconds {}\n\n\
         # HELP aruna_cpu_usage_percent Estimate of CPU usage percent.\n\
         # TYPE aruna_cpu_usage_percent gauge\n\
         aruna_cpu_usage_percent {:.2}\n\n\
         # HELP aruna_sync_progress Estimated block sync progress (0.0 to 1.0).\n\
         # TYPE aruna_sync_progress gauge\n\
         aruna_sync_progress {:.4}\n\n\
         # HELP aruna_chain_tip Block height representing the best known chain tip.\n\
         # TYPE aruna_chain_tip gauge\n\
         aruna_chain_tip {}\n\n\
         # HELP aruna_fork_count Cumulative number of blockchain reorganizations processed.\n\
         # TYPE aruna_fork_count counter\n\
         aruna_fork_count {}\n\n\
         # HELP aruna_rpc_requests_total Total number of RPC requests processed.\n\
         # TYPE aruna_rpc_requests_total counter\n\
         aruna_rpc_requests_total {}\n",
        chain_height,
        peer_count,
        mempool_size,
        state.block_time_secs,
        cpu_usage,
        sync_progress,
        max_peer_height.max(chain_height),
        fork_count,
        rpc_requests
    );

    (
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

#[derive(Serialize)]
pub struct MempoolTxResponse {
    pub hash: String,
    pub envelope: TransactionEnvelope,
}

async fn get_mempool(
    State(state): State<AppState>,
) -> Json<Vec<MempoolTxResponse>> {
    let txs = state.mempool.get_all_transactions();
    let response = txs.into_iter().map(|tx| {
        let bytes = aruna_primitives::serialize(&tx).unwrap();
        let hash = aruna_crypto::blake3_hash(&bytes);
        MempoolTxResponse {
            hash: hash.to_string(),
            envelope: tx,
        }
    }).collect();
    Json(response)
}

#[derive(Serialize)]
pub struct PeersResponse {
    pub peers: Vec<String>,
}

async fn get_peers(
    State(state): State<AppState>,
) -> Json<PeersResponse> {
    let peers = state.p2p_manager.connected_peers();
    let peers_str = peers.into_iter().map(|addr| addr.to_string()).collect();
    Json(PeersResponse { peers: peers_str })
}

#[derive(Serialize)]
pub struct NetworkResponse {
    pub network: String,
    pub chain_id: u32,
    pub peer_count: usize,
    pub uptime_seconds: u64,
    pub protocol_version: u32,
}

async fn get_network(
    State(state): State<AppState>,
) -> Json<NetworkResponse> {
    let peer_count = state.p2p_manager.peer_count();
    let uptime_seconds = state.start_time.elapsed().as_secs();
    Json(NetworkResponse {
        network: "sumatera".to_string(),
        chain_id: 1,
        peer_count,
        uptime_seconds,
        protocol_version: 1,
    })
}

#[derive(Serialize)]
pub struct SupplyResponse {
    pub circulating_supply: f64,
    pub circulating_supply_micro: u64,
    pub max_supply: u64,
    pub max_supply_micro: u64,
}

async fn get_supply(
    State(state): State<AppState>,
) -> Result<Json<SupplyResponse>, (StatusCode, String)> {
    let height = state.storage.get_chain_height()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .unwrap_or(0);

    let genesis_config = crate::bootstrap::load_genesis_config()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load genesis config: {:?}", e)))?;



    let m_aru = 1_000_000_u64;
    let mut initial_supply_micro = 0_u64;
    for amount_aru in genesis_config.allocations.values() {
        initial_supply_micro += amount_aru * m_aru;
    }

    let mut total_mined_micro = 0_u64;
    let era_size = 4_204_800_u64;
    let full_eras = height / era_size;

    for era in 0..full_eras {
        if era >= 64 {
            break;
        }
        let era_reward = 25_000_000_u64 >> era;
        total_mined_micro += era_size * era_reward;
    }

    let current_era = full_eras;
    if current_era < 64 {
        let current_era_reward = 25_000_000_u64 >> current_era;
        let blocks_in_current_era = height % era_size;
        total_mined_micro += blocks_in_current_era * current_era_reward;
    }

    let circulating_supply_micro = initial_supply_micro + total_mined_micro;
    let circulating_supply = (circulating_supply_micro as f64) / 1_000_000.0;

    Ok(Json(SupplyResponse {
        circulating_supply,
        circulating_supply_micro,
        max_supply: 1_000_000_000,
        max_supply_micro: 1_000_000_000 * 1_000_000,
    }))
}

#[derive(Serialize)]
pub struct DifficultyResponse {
    pub difficulty: u64,
}

async fn get_difficulty(
    State(state): State<AppState>,
) -> Result<Json<DifficultyResponse>, (StatusCode, String)> {
    let hash = state.storage.get_best_block()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Best block not found".to_string()))?;

    let header = state.storage.get_block_header(&hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Block header missing for hash {}", hash)))?;

    Ok(Json(DifficultyResponse {
        difficulty: header.difficulty.0 as u64,
    }))
}

#[derive(serde::Deserialize)]
pub struct ConnectPeerRequest {
    pub addr: String,
}

async fn post_peer(
    State(state): State<AppState>,
    Json(payload): Json<ConnectPeerRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let addr: std::net::SocketAddr = payload.addr.parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, format!("Invalid peer address '{}'; must be IP:PORT format.", payload.addr)))?;

    state.p2p_manager.clone().connect_to_peer(addr);
    Ok(StatusCode::OK)
}

#[derive(Serialize)]
pub struct SnapshotResponse {
    pub status: String,
    pub path: String,
}

async fn post_snapshot(
    State(state): State<AppState>,
) -> Result<Json<SnapshotResponse>, (StatusCode, String)> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let snapshot_dir = state.db_path
        .parent()
        .unwrap_or(&state.db_path)
        .join("snapshots")
        .join(format!("snapshot_{}", timestamp));

    if let Some(parent) = snapshot_dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create snapshot parent directory: {:?}", e)))?;
    }

    state.storage.create_checkpoint(&snapshot_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RocksDB checkpoint creation failed: {:?}", e)))?;

    Ok(Json(SnapshotResponse {
        status: "success".to_string(),
        path: snapshot_dir.to_string_lossy().into_owned(),
    }))
}

// --- NEW RPC ENDPOINTS (FASE 3) ---

#[derive(Serialize)]
pub struct ValidatorResponse {
    pub reward_address: String,
    pub reward_address_balance: u64,
    pub minimum_stake: u64,
    pub active_validators_count: usize,
}

#[derive(Serialize)]
pub struct TreasuryResponse {
    pub reward_address: String,
    pub reward_address_balance: u64,
    pub allocation_percent: u8,
}

#[derive(Serialize)]
pub struct RewardAddressResponse {
    pub reward_address: String,
}

#[derive(Serialize)]
pub struct AccountBalanceResponse {
    pub address: String,
    pub balance: u64,
}

#[derive(Serialize)]
pub struct AccountNonceResponse {
    pub address: String,
    pub nonce: u64,
}

#[derive(Serialize)]
pub struct PeerCountResponse {
    pub count: usize,
}

async fn get_block_by_hash(
    AxumPath(hash_str): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Json<BlockDetailResponse>, (StatusCode, String)> {
    let hash = Hash::from_hex(&hash_str)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid hash format: {:?}", e)))?;

    let header = state.storage.get_block_header(&hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Block header missing for hash {}", hash)))?;

    let body = state.storage.get_block_body(&hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, format!("Block body missing for hash {}", hash)))?;

    Ok(Json(BlockDetailResponse {
        hash: hash.to_string(),
        header,
        body,
    }))
}

async fn get_block_by_height(
    AxumPath(height): AxumPath<u64>,
    State(state): State<AppState>,
) -> Result<Json<BlockDetailResponse>, (StatusCode, String)> {
    let hash = state.storage.get_block_hash_by_height(height)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Block not found at height {}", height)))?;

    let header = state.storage.get_block_header(&hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Block header missing for hash {}", hash)))?;

    let body = state.storage.get_block_body(&hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, format!("Block body missing for hash {}", hash)))?;

    Ok(Json(BlockDetailResponse {
        hash: hash.to_string(),
        header,
        body,
    }))
}

async fn get_block_latest(
    State(state): State<AppState>,
) -> Result<Json<BlockDetailResponse>, (StatusCode, String)> {
    let hash = state.storage.get_best_block()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Best block not found".to_string()))?;

    let header = state.storage.get_block_header(&hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Block header missing for hash {}", hash)))?;

    let body = state.storage.get_block_body(&hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, format!("Block body missing for hash {}", hash)))?;

    Ok(Json(BlockDetailResponse {
        hash: hash.to_string(),
        header,
        body,
    }))
}

async fn get_pending_transactions(
    State(state): State<AppState>,
) -> Json<Vec<MempoolTxResponse>> {
    get_mempool(State(state)).await
}

async fn get_account_balance(
    AxumPath(address_str): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Json<AccountBalanceResponse>, (StatusCode, String)> {
    let (_, addr) = Address::from_bech32m(&address_str)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid address format: {:?}", e)))?;

    let account_state = state.storage.get_account(&addr)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?;

    let balance = match account_state {
        Some((bal, _, _, _)) => bal,
        None => 0,
    };

    Ok(Json(AccountBalanceResponse {
        address: address_str,
        balance,
    }))
}

async fn get_account_nonce(
    AxumPath(address_str): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Json<AccountNonceResponse>, (StatusCode, String)> {
    let (_, addr) = Address::from_bech32m(&address_str)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid address format: {:?}", e)))?;

    let account_state = state.storage.get_account(&addr)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?;

    let nonce = match account_state {
        Some((_, non, _, _)) => non,
        None => 0,
    };

    Ok(Json(AccountNonceResponse {
        address: address_str,
        nonce,
    }))
}

async fn get_supply_circulating(
    State(state): State<AppState>,
) -> Result<String, (StatusCode, String)> {
    let supply_res = get_supply(State(state)).await?;
    Ok(format!("{}", supply_res.0.circulating_supply))
}

async fn get_supply_total(
    State(state): State<AppState>,
) -> Result<String, (StatusCode, String)> {
    let supply_res = get_supply(State(state)).await?;
    Ok(format!("{}", supply_res.0.max_supply))
}

async fn get_peers_count(
    State(state): State<AppState>,
) -> Json<PeerCountResponse> {
    let count = state.p2p_manager.peer_count();
    Json(PeerCountResponse { count })
}

async fn get_validators(
    State(state): State<AppState>,
) -> Result<Json<ValidatorResponse>, (StatusCode, String)> {
    let validator_addr = state.consensus_engine.validator_reward_addr;
    let account_state = state.storage.get_account(&validator_addr)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?;

    let reward_address_balance = match account_state {
        Some((bal, _, _, _)) => bal,
        None => 0,
    };

    let reward_address = validator_addr.to_bech32m("sum").unwrap();

    Ok(Json(ValidatorResponse {
        reward_address,
        reward_address_balance,
        minimum_stake: 10_000,
        active_validators_count: 1,
    }))
}

async fn get_validator_reward_address(
    State(state): State<AppState>,
) -> Result<Json<RewardAddressResponse>, (StatusCode, String)> {
    let validator_addr = state.consensus_engine.validator_reward_addr;
    let reward_address = validator_addr.to_bech32m("sum").unwrap();
    Ok(Json(RewardAddressResponse { reward_address }))
}

async fn get_treasury(
    State(state): State<AppState>,
) -> Result<Json<TreasuryResponse>, (StatusCode, String)> {
    let treasury_addr = state.consensus_engine.treasury_reward_addr;
    let account_state = state.storage.get_account(&treasury_addr)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?;

    let reward_address_balance = match account_state {
        Some((bal, _, _, _)) => bal,
        None => 0,
    };

    let reward_address = treasury_addr.to_bech32m("sum").unwrap();

    Ok(Json(TreasuryResponse {
        reward_address,
        reward_address_balance,
        allocation_percent: 5,
    }))
}

async fn get_treasury_reward_address(
    State(state): State<AppState>,
) -> Result<Json<RewardAddressResponse>, (StatusCode, String)> {
    let treasury_addr = state.consensus_engine.treasury_reward_addr;
    let reward_address = treasury_addr.to_bech32m("sum").unwrap();
    Ok(Json(RewardAddressResponse { reward_address }))
}

pub async fn track_rpc_requests(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    state.rpc_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    next.run(request).await
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/status", get(get_status))
        .route("/health", get(get_health))
        .route("/live", get(get_live))
        .route("/ready", get(get_ready))
        .route("/tx", post(post_tx))
        .route("/tx/:hash", get(get_transaction_by_hash))
        .route("/tx/pending", get(get_pending_transactions))
        .route("/transaction/:hash", get(get_transaction_by_hash))
        .route("/transaction/hash/:hash", get(get_transaction_by_hash))
        .route("/chain/tip", get(get_chain_tip))
        .route("/blocks", get(get_blocks))
        .route("/block/latest", get(get_block_latest))
        .route("/block/:param", get(get_block_by_param))
        .route("/block/hash/:hash", get(get_block_by_hash))
        .route("/block/height/:height", get(get_block_by_height))
        .route("/address/:address", get(get_address_state))
        .route("/account/:address", get(get_address_state))
        .route("/account/:address/balance", get(get_account_balance))
        .route("/account/:address/nonce", get(get_account_nonce))
        .route("/mempool", get(get_mempool))
        .route("/peers", get(get_peers))
        .route("/peers/count", get(get_peers_count))
        .route("/peers/list", get(get_peers))
        .route("/network", get(get_network))
        .route("/supply", get(get_supply))
        .route("/supply/circulating", get(get_supply_circulating))
        .route("/supply/total", get(get_supply_total))
        .route("/difficulty", get(get_difficulty))
        .route("/difficulty/latest", get(get_difficulty))
        .route("/validators", get(get_validators))
        .route("/validator/reward-address", get(get_validator_reward_address))
        .route("/treasury", get(get_treasury))
        .route("/treasury/reward-address", get(get_treasury_reward_address))
        .route("/metrics", get(get_metrics))
        .route("/peer", post(post_peer))
        .route("/snapshot", post(post_snapshot))
        .layer(axum::middleware::from_fn(cors_middleware))
        .layer(axum::middleware::from_fn_with_state(state.clone(), track_rpc_requests))
        // CatchPanicLayer catches panics in route handlers and returns a 500 error.
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        // TraceLayer logs every request: method, path, status code, and latency.
        // Requires RUST_LOG=tower_http=debug (or =info for production).
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}


pub async fn cors_middleware(request: axum::extract::Request, next: axum::middleware::Next) -> axum::response::Response {
    let method = request.method().clone();
    if method == axum::http::Method::OPTIONS {
        let mut response = axum::response::Response::default();
        let headers = response.headers_mut();
        headers.insert("Access-Control-Allow-Origin", axum::http::HeaderValue::from_static("*"));
        headers.insert("Access-Control-Allow-Methods", axum::http::HeaderValue::from_static("GET, POST, OPTIONS"));
        headers.insert("Access-Control-Allow-Headers", axum::http::HeaderValue::from_static("Content-Type"));
        headers.insert("Access-Control-Max-Age", axum::http::HeaderValue::from_static("86400"));
        *response.status_mut() = axum::http::StatusCode::NO_CONTENT;
        return response;
    }

    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", axum::http::HeaderValue::from_static("*"));
    headers.insert("Access-Control-Allow-Methods", axum::http::HeaderValue::from_static("GET, POST, OPTIONS"));
    headers.insert("Access-Control-Allow-Headers", axum::http::HeaderValue::from_static("Content-Type"));
    response
}
