use std::sync::Arc;
use std::path::PathBuf;
use std::time::SystemTime;
use aruna_primitives::{Address, Hash, BlockHeader, BlockBody, Difficulty};
use aruna_storage::Storage;
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
    path.push(format!("aruna_rpc_endpoints_{}", nanos));
    path
}

// Async minimal TcpStream-based HTTP client
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
async fn test_all_new_rpc_endpoints() {
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

    // Populate a test account (miner_addr) in database state
    storage.put_account(&miner_addr, 500_000_000, 5, &Hash::zero(), &Hash::zero()).unwrap();

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

    let app = aruna_node::rpc::build_router(app_state);
    let rpc_port = 8495;
    let rpc_addr = format!("127.0.0.1:{}", rpc_port);
    let listener = tokio::net::TcpListener::bind(&rpc_addr).await.unwrap();
    let rpc_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Wait a brief moment for RPC server to spin up
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 1. Verify Blocks endpoints
    // GET /block/latest
    let body = send_rpc_request(rpc_port, "GET", "/block/latest", None).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["hash"], genesis_hash.to_string());

    // GET /block/height/0
    let body = send_rpc_request(rpc_port, "GET", "/block/height/0", None).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["hash"], genesis_hash.to_string());

    // GET /block/hash/:hash
    let body = send_rpc_request(rpc_port, "GET", &format!("/block/hash/{}", genesis_hash), None).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["hash"], genesis_hash.to_string());

    // 2. Verify Accounts endpoints
    let miner_bech32 = miner_addr.to_bech32m("sum").unwrap();
    
    // GET /account/:address/balance
    let body = send_rpc_request(rpc_port, "GET", &format!("/account/{}/balance", miner_bech32), None).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["balance"].as_u64().unwrap(), 500_000_000);

    // GET /account/:address/nonce
    let body = send_rpc_request(rpc_port, "GET", &format!("/account/{}/nonce", miner_bech32), None).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["nonce"].as_u64().unwrap(), 5);

    // 3. Verify Supply endpoints
    // GET /supply/circulating
    let body = send_rpc_request(rpc_port, "GET", "/supply/circulating", None).await.unwrap();
    println!("CIRCULATING SUPPLY BODY: '{}'", body);
    let circulating: f64 = body.parse().unwrap();
    assert!(circulating > 0.0);

    // GET /supply/total
    let body = send_rpc_request(rpc_port, "GET", "/supply/total", None).await.unwrap();
    let total: u64 = body.parse().unwrap();
    assert_eq!(total, 1_000_000_000);

    // 4. Verify Peers endpoints
    // GET /peers/count
    let body = send_rpc_request(rpc_port, "GET", "/peers/count", None).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["count"].as_u64().unwrap(), 0);

    // 5. Verify Validators endpoints
    // GET /validators
    let body = send_rpc_request(rpc_port, "GET", "/validators", None).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["reward_address"], validator_addr.to_bech32m("sum").unwrap());
    assert_eq!(json["minimum_stake"].as_u64().unwrap(), 10_000);

    // GET /validator/reward-address
    let body = send_rpc_request(rpc_port, "GET", "/validator/reward-address", None).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["reward_address"], validator_addr.to_bech32m("sum").unwrap());

    // 6. Verify Treasury endpoints
    // GET /treasury
    let body = send_rpc_request(rpc_port, "GET", "/treasury", None).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["reward_address"], treasury_addr.to_bech32m("sum").unwrap());
    assert_eq!(json["allocation_percent"].as_u64().unwrap(), 5);

    // GET /treasury/reward-address
    let body = send_rpc_request(rpc_port, "GET", "/treasury/reward-address", None).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["reward_address"], treasury_addr.to_bech32m("sum").unwrap());

    rpc_handle.abort();
}
