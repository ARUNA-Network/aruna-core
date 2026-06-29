use std::sync::Arc;
use std::path::PathBuf;
use std::time::SystemTime;
use aruna_primitives::{Address, Hash, BlockHeader, BlockBody, Difficulty};
use aruna_storage::{Storage, StorageBatch};
use aruna_state::StateManager;
use aruna_consensus::ConsensusEngine;
use aruna_mempool::Mempool;
use aruna_networking::P2PManager;
use aruna_node::rpc::AppState;

struct TempDirCleaner {
    path: PathBuf,
}

impl Drop for TempDirCleaner {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn temp_db_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("aruna_production_node_{}", nanos));
    path
}

// Async minimal TcpStream-based HTTP client to avoid reqwest dependency
async fn send_rpc_request(port: u16, method: &str, path: &str, body: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    use tokio::io::{AsyncWriteExt, AsyncReadExt};
    use tokio::net::TcpStream;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    let req = if let Some(b) = body {
        format!(
            "{} {} HTTP/1.1\r\n\
             Host: 127.0.0.1:{}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\r\n{}",
            method, path, port, b.len(), b
        )
    } else {
        format!(
            "{} {} HTTP/1.1\r\n\
             Host: 127.0.0.1:{}\r\n\
             Connection: close\r\n\r\n",
            method, path, port
        )
    };
    stream.write_all(req.as_bytes()).await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    if let Some(pos) = response.find("\r\n\r\n") {
        Ok(response[pos + 4..].to_string())
    } else {
        Ok(response)
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_health_and_snapshot_endpoints() {
    let temp_path = temp_db_path();
    let _cleaner = TempDirCleaner { path: temp_path.clone() };

    let db_path = temp_path.join("db");
    let peers_file = temp_path.join("peers.json");

    let storage = Storage::open(&db_path).unwrap();
    let state_manager = StateManager::new(storage.clone());

    let miner_addr = Address::from_pubkey_hash([0x01; 20]);
    let validator_addr = Address::from_pubkey_hash([0x02; 20]);
    let treasury_addr = Address::from_pubkey_hash([0x03; 20]);
    let consensus_engine = ConsensusEngine::new(
        state_manager,
        storage.clone(),
        miner_addr,
        validator_addr,
        treasury_addr,
    );
    let mempool = Arc::new(Mempool::new(100));
    let p2p_manager = Arc::new(P2PManager::new(
        storage.clone(),
        consensus_engine.clone(),
        mempool.clone(),
        9000,
        1,
        [0x00; 32],
        Some(peers_file),
    ));

    // Initialize Genesis Block 0
    let genesis_header = BlockHeader {
        version: 1,
        prev_block_hash: Hash::zero(),
        merkle_root: Hash::zero(),
        state_root: Hash::zero(),
        timestamp: 1625097600,
        difficulty: Difficulty(1),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let genesis_hash = aruna_crypto::blake3_hash(&aruna_primitives::serialize(&genesis_header).unwrap());
    storage.put_block_header(&genesis_hash, &genesis_header).unwrap();
    storage.put_block_body(&genesis_hash, &BlockBody { transactions: vec![], validator_metadata: vec![], ecosystem_metadata: vec![] }).unwrap();
    storage.put_best_block(&genesis_hash).unwrap();
    storage.put_chain_height(0).unwrap();
    storage.put_block_height_map(0, &genesis_hash).unwrap();
    storage.put_block_height_by_hash(&genesis_hash, 0).unwrap();
    storage.put_cumulative_difficulty(&genesis_hash, 0).unwrap();

    let app_state = AppState {
        storage,
        mempool,
        p2p_manager,
        consensus_engine,
        db_path: db_path.clone(),
        start_time: std::time::Instant::now(),
        block_time_secs: 30,
        rpc_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
    };

    let app = build_router(app_state);
    let rpc_port = 8395;
    let rpc_addr = format!("127.0.0.1:{}", rpc_port);
    let listener = tokio::net::TcpListener::bind(&rpc_addr).await.unwrap();
    let rpc_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Wait a brief moment for RPC server to spin up
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 1. Verify /health
    let health_body = send_rpc_request(rpc_port, "GET", "/health", None).await.unwrap();
    println!("Health response body: '{}'", health_body);
    let health_json: serde_json::Value = serde_json::from_str(&health_body).expect("Failed to parse health body as JSON");
    assert_eq!(health_json["status"], "healthy");

    // 2. Verify /snapshot (creating database checkpoint)
    let snapshot_body = send_rpc_request(rpc_port, "POST", "/snapshot", None).await.unwrap();
    println!("Snapshot response body: '{}'", snapshot_body);
    let snapshot_json: serde_json::Value = serde_json::from_str(&snapshot_body).expect("Failed to parse snapshot body as JSON");
    assert_eq!(snapshot_json["status"], "success");
    
    let checkpoint_path = PathBuf::from(snapshot_json["path"].as_str().unwrap());
    assert!(checkpoint_path.exists());
    assert!(checkpoint_path.join("CURRENT").exists()); // standard RocksDB file

    rpc_handle.abort();
}

#[test]
fn test_automatic_ledger_recovery_on_startup() {
    let test_port = 9999;
    let db_path = PathBuf::from(format!("./data_sumatera_{}", test_port));
    
    // Ensure clean state before test
    let _ = std::fs::remove_dir_all(&db_path);
    let _cleaner = TempDirCleaner { path: db_path.clone() };
    
    // Create database and initialize genesis (Block 0)
    let storage = Storage::open(&db_path).unwrap();
    let genesis_header = BlockHeader {
        version: 1,
        prev_block_hash: Hash::zero(),
        merkle_root: Hash::zero(),
        state_root: Hash::zero(),
        timestamp: 1625097600,
        difficulty: Difficulty(1),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let genesis_hash = aruna_crypto::blake3_hash(&aruna_primitives::serialize(&genesis_header).unwrap());
    storage.put_block_header(&genesis_hash, &genesis_header).unwrap();
    storage.put_block_body(&genesis_hash, &BlockBody { transactions: vec![], validator_metadata: vec![], ecosystem_metadata: vec![] }).unwrap();
    storage.put_best_block(&genesis_hash).unwrap();
    storage.put_chain_height(0).unwrap();
    storage.put_block_height_map(0, &genesis_hash).unwrap();
    storage.put_block_height_by_hash(&genesis_hash, 0).unwrap();
    storage.put_cumulative_difficulty(&genesis_hash, 0).unwrap();

    // Commit Block 1
    let block1_header = BlockHeader {
        version: 1,
        prev_block_hash: genesis_hash,
        merkle_root: Hash::zero(),
        state_root: Hash::zero(),
        timestamp: 1625097630,
        difficulty: Difficulty(1),
        nonce: 100,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let block1_hash = aruna_crypto::blake3_hash(&aruna_primitives::serialize(&block1_header).unwrap());
    storage.put_block_header(&block1_hash, &block1_header).unwrap();
    storage.put_block_body(&block1_hash, &BlockBody { transactions: vec![], validator_metadata: vec![], ecosystem_metadata: vec![] }).unwrap();
    storage.put_best_block(&block1_hash).unwrap();
    storage.put_chain_height(1).unwrap();
    storage.put_block_height_map(1, &block1_hash).unwrap();
    storage.put_block_height_by_hash(&block1_hash, 1).unwrap();
    storage.put_cumulative_difficulty(&block1_hash, 1).unwrap();

    // Commit Block 2 (Partially write/corrupted - no header in storage, but height index points to it)
    let block2_hash = Hash([0x22; 32]);
    storage.put_block_height_map(2, &block2_hash).unwrap();
    storage.put_chain_height(2).unwrap();
    storage.put_best_block(&block2_hash).unwrap();

    // Verify database height is currently 2 before bootstrap recovery
    let height_pre = storage.get_chain_height().unwrap().unwrap();
    assert_eq!(height_pre, 2);

    drop(storage); // close storage

    // Run initialize_database (which will verify integrity and trigger auto-recovery)
    let config = aruna_node::bootstrap::GenesisConfig {
        genesis: aruna_node::bootstrap::GenesisParameters {
            version: 1,
            timestamp: 1625097600,
            difficulty: 1,
            chain_id: 1,
        },
        allocations: std::collections::HashMap::new(),
    };
    
    // Copy genesis config template to temporary file just in case load_genesis_config expects config directory
    std::fs::create_dir_all("config").unwrap();
    std::fs::write("config/genesis.sumatera.toml", "
[genesis]
version = 1
timestamp = 1625097600
difficulty = 1
chain_id = 1

[allocations]
").unwrap();

    let storage_recovered = aruna_node::bootstrap::initialize_database(test_port, &config).unwrap();

    // Assert that the best block rolled back to Block 1 and height 1 automatically!
    let height_post = storage_recovered.get_chain_height().unwrap().unwrap();
    let best_post = storage_recovered.get_best_block().unwrap().unwrap();

    assert_eq!(height_post, 1);
    assert_eq!(best_post, block1_hash);
}

// Axum routes helper duplicate for E2E
fn build_router(state: AppState) -> axum::Router {
    aruna_node::rpc::build_router(state)
}
