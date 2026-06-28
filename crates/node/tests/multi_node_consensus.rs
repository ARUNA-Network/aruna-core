//! End-to-end multi-node distributed consensus integration test.
//!
//! Spawns 3 nodes in a line topology: Node A <-> Node B <-> Node C
//! Submits transactions to different nodes, mines blocks, and asserts that
//! block gossip propagates across multiple hops, resulting in 100% identical
//! chain height, block hashes, state roots, and account balances.

use aruna_primitives::{BlockBody, BlockHeader, Hash, Difficulty, Address, Nonce, TransactionPayload, TransactionEnvelope, SignatureType};
use aruna_storage::{Storage, StorageBatch};
use aruna_node::runtime::{NodeContext, rpc_loop, network_loop};
use aruna_primitives::serialize;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, Duration};

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
    path.push(format!("aruna_multinode_{}_{}", suffix, nanos));
    path
}

fn initialize_genesis_state(path: &std::path::Path, sender: &Address) -> (Storage, Hash) {
    let storage = Storage::open(path).expect("Failed to open storage");
    let mut init_batch = StorageBatch::new();
    init_batch.put_account(sender, 100_000_000, 0, &Hash::zero(), &Hash::zero());
    storage.write_batch(init_batch).unwrap();

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
    storage.put_block_height_map(0, &genesis_hash).unwrap();

    (storage, genesis_hash)
}

// ── Async HTTP client helpers ────────────────────────────────────────────────

async fn post_rpc(rpc_port: u16, path: &str, body: &str) -> Result<String, Box<dyn std::error::Error>> {
    use tokio::io::{AsyncWriteExt, AsyncReadExt};
    use tokio::net::TcpStream;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", rpc_port)).await?;
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, rpc_port, body.len(), body
    );
    stream.write_all(request.as_bytes()).await?;
    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    Ok(extract_body(&response))
}

async fn get_rpc(rpc_port: u16, path: &str) -> Result<String, Box<dyn std::error::Error>> {
    use tokio::io::{AsyncWriteExt, AsyncReadExt};
    use tokio::net::TcpStream;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", rpc_port)).await?;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        path, rpc_port
    );
    stream.write_all(request.as_bytes()).await?;
    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    Ok(extract_body(&response))
}

fn extract_body(response: &str) -> String {
    response.find("\r\n\r\n")
        .map(|i| response[i + 4..].to_string())
        .unwrap_or_else(|| response.to_string())
}

fn produce_and_commit_block(context: &Arc<NodeContext>) -> Hash {
    let txs = context.mempool.get_pending_transactions(100);
    let block = context.consensus_engine.produce_block(txs).unwrap();
    let hash = context.consensus_engine.commit_block(&block).unwrap();

    let committed_hashes: Vec<Hash> = block.body.transactions.iter().map(|tx| {
        let bytes = aruna_primitives::serialize(tx).unwrap();
        aruna_crypto::blake3_hash(&bytes)
    }).collect();
    context.mempool.remove_transactions(&committed_hashes);
    context.p2p_manager.broadcast_block(&block);

    hash
}

// ── Test Execution ────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multi_node_distributed_consensus() {
    let path_a = temp_db_path("multi_a");
    let path_b = temp_db_path("multi_b");
    let path_c = temp_db_path("multi_c");
    
    let _cleaner_a = TempDirCleaner { path: path_a.clone() };
    let _cleaner_b = TempDirCleaner { path: path_b.clone() };
    let _cleaner_c = TempDirCleaner { path: path_c.clone() };

    let keypair = aruna_crypto::Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let pkh = aruna_crypto::derive_pubkey_hash(&pubkey);
    let sender = Address::from_pubkey_hash(pkh);

    let (storage_a, _) = initialize_genesis_state(&path_a, &sender);
    let (storage_b, _) = initialize_genesis_state(&path_b, &sender);
    let (storage_c, _) = initialize_genesis_state(&path_c, &sender);

    // Node A: listening on P2P 9500, RPC 8580
    let context_a = Arc::new(NodeContext::new(
        storage_a,
        9500,
        8580,
        7777,
        path_a.clone(),
        3600, // manual mining
    ));

    // Node B: listening on P2P 9501, RPC 8581
    let context_b = Arc::new(NodeContext::new(
        storage_b,
        9501,
        8581,
        7777,
        path_b.clone(),
        3600, // manual mining
    ));

    // Node C: listening on P2P 9502, RPC 8582
    let context_c = Arc::new(NodeContext::new(
        storage_c,
        9502,
        8582,
        7777,
        path_c.clone(),
        3600, // manual mining
    ));

    // Start Node A
    network_loop::start_p2p(context_a.clone(), None).await;
    let rpc_ctx_a = context_a.clone();
    let rpc_handle_a = tokio::spawn(async move {
        let _ = rpc_loop::start_rpc_server(rpc_ctx_a).await;
    });
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Start Node B (bootstrap to Node A)
    network_loop::start_p2p(context_b.clone(), Some("127.0.0.1:9500".parse().unwrap())).await;
    let rpc_ctx_b = context_b.clone();
    let rpc_handle_b = tokio::spawn(async move {
        let _ = rpc_loop::start_rpc_server(rpc_ctx_b).await;
    });
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Start Node C (bootstrap to Node B)
    network_loop::start_p2p(context_c.clone(), Some("127.0.0.1:9501".parse().unwrap())).await;
    let rpc_ctx_c = context_c.clone();
    let rpc_handle_c = tokio::spawn(async move {
        let _ = rpc_loop::start_rpc_server(rpc_ctx_c).await;
    });

    // Wait for handshake propagation
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify initial peer counts
    println!("Node A Peers: {}", context_a.p2p_manager.peer_count());
    println!("Node B Peers: {}", context_b.p2p_manager.peer_count());
    println!("Node C Peers: {}", context_c.p2p_manager.peer_count());

    // ── Submit Transaction 1 to Node A ──────────────────────────────────────
    println!("Submitting Transaction 1 to Node A...");
    let recipient_1 = Address::from_pubkey_hash([0x11; 20]);
    let payload_1 = TransactionPayload {
        nonce: Nonce(1),
        sender,
        recipient: recipient_1,
        amount: 100_000,
        fee: 5000,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let sig_1 = keypair.sign(&serialize(&payload_1).unwrap()).to_vec();
    let tx_1 = TransactionEnvelope {
        payload: payload_1,
        signature_type: SignatureType::Ed25519,
        signature: sig_1,
        public_key: pubkey.to_vec(),
    };
    let res_1 = post_rpc(8580, "/tx", &serde_json::to_string(&tx_1).unwrap()).await.unwrap();
    assert!(res_1.contains("\"status\":\"success\""));

    // ── Submit Transaction 2 to Node C (gossips through B to A) ─────────────
    println!("Submitting Transaction 2 to Node C...");
    let recipient_2 = Address::from_pubkey_hash([0x22; 20]);
    let payload_2 = TransactionPayload {
        nonce: Nonce(2),
        sender,
        recipient: recipient_2,
        amount: 200_000,
        fee: 5000,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let sig_2 = keypair.sign(&serialize(&payload_2).unwrap()).to_vec();
    let tx_2 = TransactionEnvelope {
        payload: payload_2,
        signature_type: SignatureType::Ed25519,
        signature: sig_2,
        public_key: pubkey.to_vec(),
    };
    let res_2 = post_rpc(8582, "/tx", &serde_json::to_string(&tx_2).unwrap()).await.unwrap();
    assert!(res_2.contains("\"status\":\"success\""));

    // Wait for transaction gossip to propagate to Node A's mempool
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Mine Block 1 on Node A
    println!("Mining Block 1 on Node A...");
    let hash_1 = produce_and_commit_block(&context_a);
    println!("Block 1 mined on Node A: {:?}", hash_1);

    // Wait for block gossip to propagate through Node B to Node C
    println!("Waiting for block gossip propagation...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify all 3 nodes have same height and block hash
    let status_a: serde_json::Value = serde_json::from_str(&get_rpc(8580, "/status").await.unwrap()).unwrap();
    let status_b: serde_json::Value = serde_json::from_str(&get_rpc(8581, "/status").await.unwrap()).unwrap();
    let status_c: serde_json::Value = serde_json::from_str(&get_rpc(8582, "/status").await.unwrap()).unwrap();

    let height_a = status_a["height"].as_u64().unwrap();
    let height_b = status_b["height"].as_u64().unwrap();
    let height_c = status_c["height"].as_u64().unwrap();

    println!("Heights -> Node A: {}, Node B: {}, Node C: {}", height_a, height_b, height_c);
    assert_eq!(height_a, 1);
    assert_eq!(height_b, 1);
    assert_eq!(height_c, 1);

    let block_a: serde_json::Value = serde_json::from_str(&get_rpc(8580, "/block/1").await.unwrap()).unwrap();
    let block_b: serde_json::Value = serde_json::from_str(&get_rpc(8581, "/block/1").await.unwrap()).unwrap();
    let block_c: serde_json::Value = serde_json::from_str(&get_rpc(8582, "/block/1").await.unwrap()).unwrap();

    let hash_a = block_a["hash"].as_str().unwrap();
    let hash_b = block_b["hash"].as_str().unwrap();
    let hash_c = block_c["hash"].as_str().unwrap();

    println!("Block 1 Hashes -> Node A: {}, Node B: {}, Node C: {}", hash_a, hash_b, hash_c);
    assert_eq!(hash_a, hash_b);
    assert_eq!(hash_a, hash_c);

    let state_a = &block_a["header"]["state_root"];
    let state_b = &block_b["header"]["state_root"];
    let state_c = &block_c["header"]["state_root"];
    assert_eq!(state_a, state_b);
    assert_eq!(state_a, state_c);

    // Verify account balances are identical across all three nodes
    let sender_bech32 = sender.to_bech32m("sum").unwrap();
    let recipient_1_bech32 = recipient_1.to_bech32m("sum").unwrap();
    let recipient_2_bech32 = recipient_2.to_bech32m("sum").unwrap();

    for port in &[8580, 8581, 8582] {
        let bal_sender: serde_json::Value = serde_json::from_str(&get_rpc(*port, &format!("/address/{}", sender_bech32)).await.unwrap()).unwrap();
        let bal_r1: serde_json::Value = serde_json::from_str(&get_rpc(*port, &format!("/address/{}", recipient_1_bech32)).await.unwrap()).unwrap();
        let bal_r2: serde_json::Value = serde_json::from_str(&get_rpc(*port, &format!("/address/{}", recipient_2_bech32)).await.unwrap()).unwrap();

        assert_eq!(bal_sender["balance"].as_u64().unwrap(), 99_690_000); // 100M - 100K - 200K - 10K fees
        assert_eq!(bal_r1["balance"].as_u64().unwrap(), 100_000);
        assert_eq!(bal_r2["balance"].as_u64().unwrap(), 200_000);
    }

    rpc_handle_a.abort();
    rpc_handle_b.abort();
    rpc_handle_c.abort();
    println!("Multi-node distributed consensus integration test passed!");
}
