//! ARUNA core node runner.
//! Loads genesis configuration from toml file, initializes RocksDB storage, and verifies ledger state.

use aruna_primitives::{Block, BlockBody, BlockHeader, Hash, Address, Difficulty, TransactionEnvelope};
use aruna_storage::{Storage, StorageBatch};
use aruna_state::StateManager;
use aruna_consensus::ConsensusEngine;
use aruna_mempool::Mempool;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
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
}

#[derive(serde::Serialize)]
struct StatusResponse {
    network: String,
    height: u64,
}

#[derive(serde::Serialize)]
struct TxResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

async fn get_status(
    State(state): State<AppState>,
) -> Result<Json<StatusResponse>, (axum::http::StatusCode, String)> {
    match state.storage.get_chain_height() {
        Ok(maybe_height) => {
            let height = maybe_height.unwrap_or(0);
            Ok(Json(StatusResponse {
                network: "sumatera".to_string(),
                height,
            }))
        }
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {:?}", e),
        )),
    }
}

async fn post_tx(
    State(state): State<AppState>,
    Json(tx): Json<TransactionEnvelope>,
) -> Result<Json<TxResponse>, (axum::http::StatusCode, Json<TxResponse>)> {
    match state.mempool.add_transaction(tx, &state.storage) {
        Ok(hash) => Ok(Json(TxResponse {
            status: "success".to_string(),
            tx_hash: Some(hash.to_string()),
            message: None,
        })),
        Err(e) => Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(TxResponse {
                status: "error".to_string(),
                tx_hash: None,
                message: Some(e.to_string()),
            }),
        )),
    }
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
    if args.len() > 1 {
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
                println!("  aruna-node               Start the full node daemon (default)");
                return Ok(());
            }
            other => {
                eprintln!("Error: Unknown subcommand '{}'. Run with 'help' for usage.", other);
                std::process::exit(1);
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

    // 2. Establish data storage directory (Sumatera Testnet baseline)
    let db_path = Path::new("./data_sumatera");
    
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
        
        storage.write_batch(batch)?;
        
        // Save best/finalized metadata and chain_id
        storage.put_best_block(&genesis_hash)?;
        storage.put_chain_height(0)?;
        storage.put_finalized_block(&genesis_hash)?;
        storage.put_chain_id(config.genesis.chain_id)?;
    } else {
        println!("Genesis already initialized. Loading existing ledger state...");
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

    // 8. Start Block Producer Loop in background (Sprint A & B)
    // Produces a block every 30 seconds, validating it and persisting it to RocksDB.
    let storage_clone = storage.clone();
    let consensus_clone = consensus_engine.clone();
    
    tokio::spawn(async move {
        println!("Starting Block Producer loop (30-second interval)...");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            let current_height = match storage_clone.get_chain_height() {
                Ok(h) => h.unwrap_or(0),
                Err(e) => {
                    eprintln!("Error reading chain height: {:?}", e);
                    continue;
                }
            };
            println!("Current Height: {}", current_height);

            match consensus_clone.produce_block() {
                Ok(block) => {
                    match consensus_clone.commit_block(&block) {
                        Ok(hash) => {
                            let height = match storage_clone.get_chain_height() {
                                Ok(h) => h.unwrap_or(0),
                                Err(e) => {
                                    eprintln!("Error reading chain height: {:?}", e);
                                    continue;
                                }
                            };
                            println!("New Height: {}", height);
                            println!(
                                "Block #{} produced | Height: {} | Height={} | Hash: {}",
                                height, height, height, hash
                            );
                        }
                        Err(e) => eprintln!("Error committing block: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Error producing block: {:?}", e),
            }
        }
    });

    // 9. Start HTTP RPC Server (Sprint C & Sprint 2)
    let app_state = AppState {
        storage: storage.clone(),
        mempool,
    };

    let app = Router::new()
        .route("/status", get(get_status))
        .route("/tx", post(post_tx))
        .with_state(app_state);

    println!("Starting RPC server on 127.0.0.1:8080...");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
