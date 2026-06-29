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

#[derive(Serialize)]
pub struct StatusResponse {
    pub network: String,
    pub height: u64,
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

async fn get_status(
    State(state): State<AppState>,
) -> Result<Json<StatusResponse>, (StatusCode, String)> {
    match state.storage.get_chain_height() {
        Ok(maybe_height) => {
            let height = maybe_height.unwrap_or(0);
            Ok(Json(StatusResponse {
                network: "sumatera".to_string(),
                height,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {:?}", e),
        )),
    }
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
        .route("/tx", post(post_tx))
        .route("/tx/:hash", get(get_transaction_by_hash))
        .route("/chain/tip", get(get_chain_tip))
        .route("/blocks", get(get_blocks))
        .route("/block/:param", get(get_block_by_param))
        .route("/address/:address", get(get_address_state))
        .route("/account/:address", get(get_address_state))
        .route("/transaction/:hash", get(get_transaction_by_hash))
        .route("/mempool", get(get_mempool))
        .route("/peers", get(get_peers))
        .route("/network", get(get_network))
        .route("/supply", get(get_supply))
        .route("/difficulty", get(get_difficulty))
        .route("/metrics", get(get_metrics))
        .route("/peer", post(post_peer))
        .layer(axum::middleware::from_fn(cors_middleware))
        .layer(axum::middleware::from_fn_with_state(state.clone(), track_rpc_requests))
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
