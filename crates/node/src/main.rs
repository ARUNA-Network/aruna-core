//! ARUNA core node runner.
//! Loads genesis configuration from toml file, initializes RocksDB storage, and verifies ledger state.

use aruna_primitives::{Block, BlockBody, BlockHeader, Hash, Address, Difficulty, TransactionEnvelope};
use aruna_storage::{Storage, StorageBatch};
use aruna_state::StateManager;
use aruna_consensus::ConsensusEngine;
use aruna_mempool::Mempool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{State, Path as AxumPath, Query},
    http::StatusCode,
};

#[derive(Debug, Deserialize)]
struct GenesisConfig {
    genesis: GenesisParameters,
    allocations: HashMap<String, u64>,
}

#[derive(Debug, Deserialize)]
struct GenesisParameters {
    version: u32,
    timestamp: u64,
    difficulty: u32,
    chain_id: u32,
}

#[derive(Clone)]
struct AppState {
    storage: Storage,
    mempool: Arc<Mempool>,
    p2p_manager: Arc<aruna_networking::P2PManager>,
}

#[derive(Serialize)]
struct StatusResponse {
    network: String,
    height: u64,
}

#[derive(Serialize)]
struct TxResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Deserialize)]
struct BlocksParams {
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Serialize)]
struct TipResponse {
    hash: String,
    height: u64,
}

#[derive(Serialize)]
struct BlockSummaryResponse {
    height: u64,
    hash: String,
    prev_block_hash: String,
    merkle_root: String,
    timestamp: u64,
    difficulty: u64,
    nonce: u64,
    tx_count: usize,
}

#[derive(Serialize)]
struct BlockDetailResponse {
    hash: String,
    header: BlockHeader,
    body: BlockBody,
}

#[derive(Serialize)]
struct AddressResponse {
    address: String,
    balance: u64,
    nonce: u64,
    code_hash: String,
    storage_root: String,
}

#[derive(Serialize)]
struct TransactionResponse {
    hash: String,
    status: String, // "pending" or "committed"
    #[serde(skip_serializing_if = "Option::is_none")]
    block_height: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    block_hash: Option<String>,
    transaction: TransactionEnvelope,
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

async fn get_block_by_height(
    AxumPath(height): AxumPath<u64>,
    State(state): State<AppState>,
) -> Result<Json<BlockDetailResponse>, (StatusCode, String)> {
    let hash = state.storage.get_block_hash_by_height(height)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Block not found at height {}", height)))?;

    let header = state.storage.get_block_header(&hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {:?}", e)))?
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, format!("Block header missing for hash {}", hash)))?;

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

fn submit_transaction_cli(tx_json: &str) -> Result<String, Box<dyn std::error::Error>> {
    use std::io::{Write, Read};
    use std::net::TcpStream;

    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    let request = format!(
        "POST /tx HTTP/1.1\r\n\
         Host: 127.0.0.1:8080\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n\
         {}",
        tx_json.len(),
        tx_json
    );
    stream.write_all(request.as_bytes())?;
    
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    
    if let Some(body_start) = response.find("\r\n\r\n") {
        Ok(response[body_start + 4..].to_string())
    } else {
        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    // Check if the user is executing a CLI subcommand
    if args.len() > 1 && !args[1].starts_with("-") {
        let subcommand = &args[1];
        let db_path = Path::new("./data_sumatera");

        match subcommand.as_str() {
            "submit" => {
                if args.len() < 3 {
                    eprintln!("Error: Missing transaction JSON file. Usage: aruna-node submit <tx_json_file>");
                    std::process::exit(1);
                }
                let file_path = &args[2];
                let tx_json = match std::fs::read_to_string(file_path) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!("Error reading file '{}': {:?}", file_path, e);
                        std::process::exit(1);
                    }
                };

                // Local validation to ensure the JSON matches the schema
                if let Err(e) = serde_json::from_str::<TransactionEnvelope>(&tx_json) {
                    eprintln!("Error: Transaction file is not a valid JSON TransactionEnvelope: {:?}", e);
                    std::process::exit(1);
                }

                match submit_transaction_cli(&tx_json) {
                    Ok(response_body) => {
                        println!("{}", response_body.trim());
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("Error: Could not connect to the node RPC server at 127.0.0.1:8080.");
                        eprintln!("Details: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
            "status" | "block" | "blocks" => {
                // Open database in read-only mode for inspection subcommands
                let storage = match Storage::open_read_only(db_path) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error opening database in read-only mode: {:?}", e);
                        eprintln!("Please ensure the node has been started at least once to initialize the database.");
                        std::process::exit(1);
                    }
                };

                match subcommand.as_str() {
                    "status" => {
                        let height = storage.get_chain_height()?.unwrap_or(0);
                        let tip_hash = storage.get_best_block()?
                            .map(|h| h.to_string())
                            .unwrap_or_else(|| "none".to_string());
                        
                        println!("{{\"height\":{},\"tip\":\"{}\"}}", height, tip_hash);
                        return Ok(());
                    }
                    "block" => {
                        if args.len() < 3 {
                            eprintln!("Error: Missing block height. Usage: aruna-node block <height>");
                            std::process::exit(1);
                        }
                        let height_str = &args[2];
                        let height = match height_str.parse::<u64>() {
                            Ok(h) => h,
                            Err(_) => {
                                eprintln!("Error: Invalid block height '{}'. Must be a non-negative integer.", height_str);
                                std::process::exit(1);
                            }
                        };

                        match storage.get_block_hash_by_height(height)? {
                            Some(hash) => {
                                println!("{{\"height\":{},\"hash\":\"{}\"}}", height, hash);
                                return Ok(());
                            }
                            None => {
                                eprintln!("Error: Block not found at height {}", height);
                                std::process::exit(1);
                            }
                        }
                    }
                    "blocks" => {
                        let height = storage.get_chain_height()?.unwrap_or(0);
                        for h in 1..=height {
                            println!("{}", h);
                        }
                        return Ok(());
                    }
                    _ => unreachable!(),
                }
            }
            "help" | "-h" | "--help" => {
                println!("ARUNA Chain Inspection & Transaction CLI");
                println!("Usage:");
                println!("  aruna-node status        Display current chain height and tip block hash");
                println!("  aruna-node block <h>     Display the block hash at height <h>");
                println!("  aruna-node blocks        List all block heights from 1 to the current tip");
                println!("  aruna-node submit <file> Submit a signed transaction JSON file to the mempool");
                println!();
                println!("Daemon Options (when starting the node):");
                println!("  --p2p-port <port>        P2P listening port (default: 9000)");
                println!("  --rpc-port <port>        HTTP RPC listening port (default: 8080)");
                println!("  --peer <ip:port>         Bootstrap peer address to connect to");
                println!("  aruna-node               Start the full node daemon (default)");
                return Ok(());
            }
            other => {
                eprintln!("Error: Unknown subcommand '{}'. Run with 'help' for usage.", other);
                std::process::exit(1);
            }
        }
    }

    // Parse daemon arguments
    let mut p2p_port = 9000;
    let mut rpc_port = 8080;
    let mut peer_addr = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--p2p-port" => {
                if i + 1 < args.len() {
                    p2p_port = args[i + 1].parse::<u16>().expect("Invalid P2P port");
                    i += 2;
                } else {
                    eprintln!("Error: Missing value for --p2p-port");
                    std::process::exit(1);
                }
            }
            "--rpc-port" => {
                if i + 1 < args.len() {
                    rpc_port = args[i + 1].parse::<u16>().expect("Invalid RPC port");
                    i += 2;
                } else {
                    eprintln!("Error: Missing value for --rpc-port");
                    std::process::exit(1);
                }
            }
            "--peer" => {
                if i + 1 < args.len() {
                    let addr_str = &args[i + 1];
                    let parsed: std::net::SocketAddr = addr_str.parse()
                        .expect("Invalid peer address; must be IP:PORT format (e.g. 127.0.0.1:9000)");
                    peer_addr = Some(parsed);
                    i += 2;
                } else {
                    eprintln!("Error: Missing value for --peer");
                    std::process::exit(1);
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    println!("ARUNA Core Node starting...");

    // 1. Load genesis configuration from TOML file
    let config_path = Path::new("config/genesis.sumatera.toml");
    if !config_path.exists() {
        return Err(format!("Genesis configuration file not found at: {:?}", config_path).into());
    }
    let config_str = std::fs::read_to_string(config_path)?;
    let config: GenesisConfig = toml::from_str(&config_str)?;

    // 2. Establish data storage directory (dynamic depending on P2P port to allow local multi-node testing)
    let db_dir = if p2p_port == 9000 {
        "./data_sumatera".to_string()
    } else {
        format!("./data_sumatera_{}", p2p_port)
    };
    let db_path = Path::new(&db_dir);
    
    // 3. Open RocksDB Storage
    let storage = Storage::open(db_path)?;

    // 4. Initialize StateManager & ConsensusEngine
    let state_manager = StateManager::new(storage.clone());
    let consensus_engine = ConsensusEngine::new(state_manager.clone(), storage.clone());

    // 5. Check if Genesis Block (Height 0) is already loaded
    let best_block = storage.get_best_block()?;
    
    if best_block.is_none() {
        println!("Initializing new ledger state from genesis config...");
        let mut batch = StorageBatch::new();

        // Dynamically apply allocations from TOML
        let m_aru = 1_000_000_u64; // 1 ARU = 1,000,000 micro-ARU
        for (address_str, amount_aru) in &config.allocations {
            let (_, addr) = Address::from_bech32m(address_str)?;
            let amount_micro = amount_aru.checked_mul(m_aru).ok_or("Allocation calculation overflow")?;
            batch.put_account(&addr, amount_micro, 0, &Hash::zero(), &Hash::zero());
        }

        // Construct Genesis Block Header from TOML parameters
        let genesis_header = BlockHeader {
            version: config.genesis.version,
            prev_block_hash: Hash::zero(),
            merkle_root: Hash::zero(),
            timestamp: config.genesis.timestamp,
            difficulty: Difficulty(config.genesis.difficulty),
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        };

        let genesis_body = BlockBody {
            transactions: vec![],
            validator_metadata: vec![],
            ecosystem_metadata: vec![],
        };

        let genesis_block = Block {
            header: genesis_header,
            body: genesis_body,
        };

        // Serialize and calculate Genesis hash (BLAKE3)
        let header_bytes = aruna_primitives::serialize(&genesis_header)?;
        let genesis_hash = aruna_crypto::blake3_hash(&header_bytes);

        // Persist Block 0 data
        storage.put_block_header(&genesis_hash, &genesis_header)?;
        storage.put_block_body(&genesis_hash, &genesis_block.body)?;

        // Update Chain Metadata indexes in storage batch
        batch.put_block_height_map(0, &genesis_hash);
        batch.put_block_height_by_hash(&genesis_hash, 0);
        
        storage.write_batch(batch)?;
        
        // Save best/finalized metadata and chain_id
        storage.put_best_block(&genesis_hash)?;
        storage.put_chain_height(0)?;
        storage.put_finalized_block(&genesis_hash)?;
        storage.put_chain_id(config.genesis.chain_id)?;
    } else {
        println!("Genesis already initialized. Loading existing ledger state...");
    }

    // --- Self-Healing Index Backfill ---
    let best_height = storage.get_chain_height()?.unwrap_or(0);
    for h in 0..=best_height {
        if let Some(hash) = storage.get_block_hash_by_height(h)? {
            if storage.get_block_height_by_hash(&hash)?.is_none() {
                println!("Backfilling block hash to height index for block #{} ({})", h, hash);
                storage.put_block_height_by_hash(&hash, h)?;
            }
        }
    }

    // 6. Print Node Successful initialization banner
    println!("\nARUNA Node Started");
    println!("Network : Sumatera Testnet");
    println!("Height  : 0");
    println!("Genesis : Loaded");
    println!("Storage : Opened");
    println!("State   : Initialized\n");

    // 7. Initialize Mempool
    let mempool = Arc::new(Mempool::new(50000));

    // 8. Initialize P2P Networking Manager
    let p2p_manager = Arc::new(aruna_networking::P2PManager::new(
        storage.clone(),
        consensus_engine.clone(),
        mempool.clone(),
        p2p_port,
        config.genesis.chain_id,
    ));

    // Start P2P Server
    p2p_manager.clone().start_server();

    // Connect to bootstrap peer if provided
    if let Some(peer) = peer_addr {
        p2p_manager.clone().connect_to_peer(peer);
    }

    // 9. Start Block Producer Loop in background (Sprint A & B)
    // Produces a block every 30 seconds, validating it and persisting it to RocksDB.
    let storage_clone = storage.clone();
    let consensus_clone = consensus_engine.clone();
    let mempool_clone = mempool.clone();
    let p2p_manager_clone = p2p_manager.clone();
    
    tokio::spawn(async move {
        println!("Starting Block Producer loop (30-second interval)...");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            
            // Fetch pending transactions from mempool
            let txs = mempool_clone.get_pending_transactions(100);
            
            let current_height = match storage_clone.get_chain_height() {
                Ok(h) => h.unwrap_or(0),
                Err(e) => {
                    eprintln!("Error reading chain height: {:?}", e);
                    continue;
                }
            };
            println!("Current Height: {}", current_height);

            match consensus_clone.produce_block(txs) {
                Ok(block) => {
                    let tx_count = block.body.transactions.len();
                    match consensus_clone.commit_block(&block) {
                        Ok(hash) => {
                            // Evict committed transactions from the mempool
                            let committed_hashes: Vec<Hash> = block.body.transactions.iter().map(|tx| {
                                let bytes = aruna_primitives::serialize(tx).unwrap();
                                aruna_crypto::blake3_hash(&bytes)
                            }).collect();
                            mempool_clone.remove_transactions(&committed_hashes);
                            
                            // Broadcast the block to P2P peers!
                            p2p_manager_clone.broadcast_block(&block);
                            
                            let height = match storage_clone.get_chain_height() {
                                Ok(h) => h.unwrap_or(0),
                                Err(e) => {
                                    eprintln!("Error reading chain height: {:?}", e);
                                    continue;
                                }
                            };
                            println!("New Height: {}", height);
                            println!(
                                "Block #{} produced with {} transactions | Height: {} | Height={} | Hash: {}",
                                height, tx_count, height, height, hash
                            );
                        }
                        Err(e) => eprintln!("Error committing block: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Error producing block: {:?}", e),
            }
        }
    });

    // 10. Start HTTP RPC Server (Sprint C & Sprint 2)
    let app_state = AppState {
        storage: storage.clone(),
        mempool,
        p2p_manager: p2p_manager.clone(),
    };

    let app = Router::new()
        .route("/status", get(get_status))
        .route("/tx", post(post_tx))
        .route("/chain/tip", get(get_chain_tip))
        .route("/blocks", get(get_blocks))
        .route("/block/:height", get(get_block_by_height))
        .route("/address/:address", get(get_address_state))
        .route("/transaction/:hash", get(get_transaction_by_hash))
        .with_state(app_state);

    let rpc_addr = format!("127.0.0.1:{}", rpc_port);
    println!("Starting RPC server on {}...", rpc_addr);
    let listener = tokio::net::TcpListener::bind(&rpc_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
