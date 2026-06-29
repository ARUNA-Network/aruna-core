//! Wallet End-to-End integration test for the ARUNA Network.
//! Verifies: Create Wallet -> Sign -> Broadcast -> Mempool -> Block -> Balance Updated -> Explorer.

use aruna_primitives::{
    BlockBody, BlockHeader, Hash, Difficulty, Address, Nonce,
    TransactionPayload, TransactionEnvelope, SignatureType, serialize
};
use aruna_storage::{Storage, StorageBatch};
use aruna_node::runtime::{NodeContext, rpc_loop};
use aruna_crypto::{Ed25519Keypair, derive_pubkey_hash};
use std::path::PathBuf;
use std::time::{SystemTime, Duration};
use std::sync::Arc;

struct TempDirCleaner {
    path: PathBuf,
}

impl Drop for TempDirCleaner {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn temp_db_path(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("aruna_wallet_e2e_{}_{}", suffix, nanos));
    path
}

fn initialize_genesis_state(path: &std::path::Path, sender: &Address) -> (Storage, Hash) {
    let storage = Storage::open(path).expect("Failed to open storage");

    // Pre-fund sender in genesis
    let mut init_batch = StorageBatch::new();
    init_batch.put_account(sender, 1_000_000, 0, &Hash::zero(), &Hash::zero());
    storage.write_batch(init_batch).unwrap();

    // Genesis Block
    let genesis_header = BlockHeader {
        version: 1,
        prev_block_hash: Hash::zero(),
        merkle_root: Hash::zero(),
        state_root: Hash::zero(),
        timestamp: 1782252000,
        difficulty: Difficulty(504381424),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let genesis_body = BlockBody {
        transactions: vec![],
        validator_metadata: vec![],
        ecosystem_metadata: vec![],
    };
    let genesis_bytes = serialize(&genesis_header).unwrap();
    let genesis_hash = aruna_crypto::blake3_hash(&genesis_bytes);

    storage.put_block_header(&genesis_hash, &genesis_header).unwrap();
    storage.put_block_body(&genesis_hash, &genesis_body).unwrap();
    storage.put_best_block(&genesis_hash).unwrap();
    storage.put_chain_height(0).unwrap();
    storage.put_cumulative_difficulty(&genesis_hash, 0).unwrap();
    storage.put_block_height_by_hash(&genesis_hash, 0).unwrap();

    (storage, genesis_hash)
}

// Async minimal TcpStream-based HTTP client to avoid blocking tokio thread pools
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
async fn test_wallet_end_to_end_flow() {
    let path = temp_db_path("node");
    let _cleaner = TempDirCleaner { path: path.clone() };

    // 1. Create Wallet (Generate Keys)
    let sender_keypair = Ed25519Keypair::generate();
    let sender_pubkey = sender_keypair.public_key_bytes();
    let sender_address = Address::from_pubkey_hash(derive_pubkey_hash(&sender_pubkey));

    let recipient_keypair = Ed25519Keypair::generate();
    let recipient_pubkey = recipient_keypair.public_key_bytes();
    let recipient_address = Address::from_pubkey_hash(derive_pubkey_hash(&recipient_pubkey));

    // Initialize state
    let (storage, _genesis_hash) = initialize_genesis_state(&path, &sender_address);

    // Setup Node Context running on RPC port 8185
    let rpc_port = 8185;
    let context = Arc::new(NodeContext::new(storage.clone(), 9105, rpc_port, 7777, path.clone(), 30));

    // Start HTTP RPC server task
    let rpc_ctx = context.clone();
    let rpc_handle = tokio::spawn(async move {
        let _ = rpc_loop::start_rpc_server(rpc_ctx).await;
    });

    // Wait a brief moment for RPC server to bind
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 2. Sign Transaction
    let payload = TransactionPayload {
        nonce: Nonce(1),
        sender: sender_address,
        recipient: recipient_address,
        amount: 50_000,
        fee: 5000,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let payload_bytes = serialize(&payload).unwrap();
    let signature = sender_keypair.sign(&payload_bytes).to_vec();

    let tx_envelope = TransactionEnvelope {
        payload,
        signature_type: SignatureType::Ed25519,
        signature,
        public_key: sender_pubkey.to_vec(),
    };

    // Serialize envelope to JSON
    let tx_json = serde_json::to_string(&tx_envelope).unwrap();

    // 3. Broadcast transaction via RPC (POST /tx)
    println!("Broadcasting signed transaction to Node RPC...");
    let response_body = send_rpc_request(rpc_port, "POST", "/tx", Some(&tx_json))
        .await
        .expect("Failed to send broadcast request");

    println!("RPC Broadcast response: {}", response_body);
    let submit_res: serde_json::Value = serde_json::from_str(&response_body)
        .expect("Failed to parse broadcast response JSON");

    assert_eq!(submit_res["status"], "success");
    let tx_hash_str = submit_res["tx_hash"].as_str().expect("Missing tx_hash in response");
    let tx_hash = Hash::from_slice(&hex::decode(tx_hash_str).unwrap()).unwrap();

    // 4. Mempool Check
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(context.mempool.contains(&tx_hash), "Transaction must reside in node mempool");

    // 5. Block Packaging: Produce and commit block manually
    println!("Manually triggering block production for mempool transactions...");
    let txs = context.mempool.get_pending_transactions(100);
    assert!(!txs.is_empty(), "Mempool must contain the transaction");
    let block = context.consensus_engine.produce_block(txs).unwrap();
    let committed_hash = context.consensus_engine.commit_block(&block).unwrap();

    // Evict committed transactions from the mempool
    let committed_hashes: Vec<Hash> = block.body.transactions.iter().map(|tx| {
        let bytes = serialize(tx).unwrap();
        aruna_crypto::blake3_hash(&bytes)
    }).collect();
    context.mempool.remove_transactions(&committed_hashes);

    println!("Committed Block #1 with hash: {}", committed_hash);
    let height = storage.get_chain_height().unwrap().unwrap();
    assert_eq!(height, 1);

    // 6. Balance Updated Check
    println!("Querying updated balances via RPC address status...");
    let recipient_state_body = send_rpc_request(rpc_port, "GET", &format!("/address/{}", recipient_address.to_bech32m("sum").unwrap()), None)
        .await
        .expect("Failed to fetch recipient address status");
    let recipient_state: serde_json::Value = serde_json::from_str(&recipient_state_body).unwrap();
    assert_eq!(recipient_state["balance"].as_u64().unwrap(), 50_000);

    let sender_state_body = send_rpc_request(rpc_port, "GET", &format!("/address/{}", sender_address.to_bech32m("sum").unwrap()), None)
        .await
        .expect("Failed to fetch sender address status");
    let sender_state: serde_json::Value = serde_json::from_str(&sender_state_body).unwrap();
    assert_eq!(sender_state["balance"].as_u64().unwrap(), 1_000_000 - 50_000 - 5000);
    assert_eq!(sender_state["nonce"].as_u64().unwrap(), 1);

    // 7. Explorer Check: Verify Transaction lookup by hash
    println!("Verifying explorer lookup via RPC transaction endpoint...");
    let tx_lookup_body = send_rpc_request(rpc_port, "GET", &format!("/transaction/{}", tx_hash_str), None)
        .await
        .expect("Failed to query transaction status");
    let tx_lookup: serde_json::Value = serde_json::from_str(&tx_lookup_body).unwrap();
    assert_eq!(tx_lookup["status"], "committed");
    assert_eq!(tx_lookup["block_height"].as_u64().unwrap(), 1);

    // 8. Observability Check: Verify /metrics endpoint is populated and correct (Prometheus format)
    println!("Verifying /metrics endpoint...");
    let metrics_body = send_rpc_request(rpc_port, "GET", "/metrics", None)
        .await
        .expect("Failed to fetch /metrics");
    
    assert!(metrics_body.contains("aruna_block_height 1"), "metrics should contain aruna_block_height 1");
    assert!(metrics_body.contains("aruna_mempool_size 0"), "metrics should contain aruna_mempool_size 0");
    assert!(metrics_body.contains("aruna_peer_count 0"), "metrics should contain aruna_peer_count 0");
    assert!(metrics_body.contains("aruna_block_time_seconds 30"), "metrics should contain aruna_block_time_seconds 30");
    assert!(metrics_body.contains("aruna_fork_count 0"), "metrics should contain aruna_fork_count 0");
    assert!(metrics_body.contains("aruna_rpc_requests_total"), "metrics should contain aruna_rpc_requests_total");

    // Teardown Axum RPC server
    rpc_handle.abort();
    println!("Wallet End-to-End flow successful!");
}
