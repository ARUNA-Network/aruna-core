//! End-to-end network-level fork simulation and choice integration test.
//!
//! 1. Starts Node A and Node B independently starting from same genesis (Network Split).
//! 2. Submits 1 transaction to Node A -> mines 1 block (Height 1).
//! 3. Submits 2 transactions to Node B -> mines 2 blocks (Height 2).
//! 4. Asserts that Node A is at height 1, Node B is at height 2, with different tips (Fork).
//! 5. Reconnects Node B to Node A via the `/peer` RPC endpoint.
//! 6. Asserts that both nodes perform P2P handshake, block sync, reorg,
//!    and settle on the heavier canonical chain (height 2) with identical state roots.

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
    path.push(format!("aruna_fork_simulation_{}_{}", suffix, nanos));
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
async fn test_network_fork_simulation() {
    let path_a = temp_db_path("fork_a");
    let path_b = temp_db_path("fork_b");
    let _cleaner_a = TempDirCleaner { path: path_a.clone() };
    let _cleaner_b = TempDirCleaner { path: path_b.clone() };

    let keypair = aruna_crypto::Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let pkh = aruna_crypto::derive_pubkey_hash(&pubkey);
    let sender = Address::from_pubkey_hash(pkh);

    let (storage_a, _) = initialize_genesis_state(&path_a, &sender);
    let (storage_b, _) = initialize_genesis_state(&path_b, &sender);

    // Node A: block_time = 3600 (manual mining only)
    let context_a = Arc::new(NodeContext::new(
        storage_a,
        9300, // p2p port
        8380, // rpc port
        7777, // chain id
        path_a.clone(),
        3600, // 1-hour block time
    ));

    // Node B: block_time = 3600 (manual mining only)
    let context_b = Arc::new(NodeContext::new(
        storage_b,
        9301, // p2p port
        8381, // rpc port
        7777, // chain id
        path_b.clone(),
        3600, // 1-hour block time
    ));

    // Start Node A loops (no peers connected)
    network_loop::start_p2p(context_a.clone(), None).await;
    let rpc_ctx_a = context_a.clone();
    let rpc_handle_a = tokio::spawn(async move {
        let _ = rpc_loop::start_rpc_server(rpc_ctx_a).await;
    });

    // Start Node B loops (no peers connected - Network Split!)
    network_loop::start_p2p(context_b.clone(), None).await;
    let rpc_ctx_b = context_b.clone();
    let rpc_handle_b = tokio::spawn(async move {
        let _ = rpc_loop::start_rpc_server(rpc_ctx_b).await;
    });

    // Wait a brief moment for servers to bind
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // 1. Submit transaction to Node A -> Branch A (mines Block A1)
    println!("Submitting Transaction to Node A...");
    let recipient_a = Address::from_pubkey_hash([0x0a; 20]);
    let payload_a = TransactionPayload {
        nonce: Nonce(1),
        sender,
        recipient: recipient_a,
        amount: 100_000,
        fee: 5000,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let payload_a_bytes = serialize(&payload_a).unwrap();
    let sig_a = keypair.sign(&payload_a_bytes).to_vec();
    let tx_a = TransactionEnvelope {
        payload: payload_a,
        signature_type: SignatureType::Ed25519,
        signature: sig_a,
        public_key: pubkey.to_vec(),
    };
    let tx_a_json = serde_json::to_string(&tx_a).unwrap();
    let res_a = post_rpc(8380, "/tx", &tx_a_json).await.unwrap();
    assert!(res_a.contains("\"status\":\"success\""));

    // Produce Block A1 manually
    produce_and_commit_block(&context_a);

    // 2. Submit 2 sequential transactions to Node B -> Branch B (mines Block B1 and Block B2)
    println!("Submitting Transaction 1 to Node B...");
    let recipient_b = Address::from_pubkey_hash([0x0b; 20]);
    let payload_b1 = TransactionPayload {
        nonce: Nonce(1),
        sender,
        recipient: recipient_b,
        amount: 200_000,
        fee: 5000,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let sig_b1 = keypair.sign(&serialize(&payload_b1).unwrap()).to_vec();
    let tx_b1 = TransactionEnvelope {
        payload: payload_b1,
        signature_type: SignatureType::Ed25519,
        signature: sig_b1,
        public_key: pubkey.to_vec(),
    };
    let res_b1 = post_rpc(8381, "/tx", &serde_json::to_string(&tx_b1).unwrap()).await.unwrap();
    assert!(res_b1.contains("\"status\":\"success\""));

    // Produce Block B1 manually
    produce_and_commit_block(&context_b);

    println!("Submitting Transaction 2 to Node B...");
    let payload_b2 = TransactionPayload {
        nonce: Nonce(2),
        sender,
        recipient: recipient_b,
        amount: 300_000,
        fee: 5000,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let sig_b2 = keypair.sign(&serialize(&payload_b2).unwrap()).to_vec();
    let tx_b2 = TransactionEnvelope {
        payload: payload_b2,
        signature_type: SignatureType::Ed25519,
        signature: sig_b2,
        public_key: pubkey.to_vec(),
    };
    let res_b2 = post_rpc(8381, "/tx", &serde_json::to_string(&tx_b2).unwrap()).await.unwrap();
    assert!(res_b2.contains("\"status\":\"success\""));

    // Produce Block B2 manually
    produce_and_commit_block(&context_b);

    // Verify current status (Forks present!)
    let status_a_str = get_rpc(8380, "/status").await.unwrap();
    let status_b_str = get_rpc(8381, "/status").await.unwrap();
    let status_a: serde_json::Value = serde_json::from_str(&status_a_str).unwrap();
    let status_b: serde_json::Value = serde_json::from_str(&status_b_str).unwrap();

    let height_a = status_a["height"].as_u64().unwrap();
    let height_b = status_b["height"].as_u64().unwrap();
    println!("Forks established -> Node A Height: {}, Node B Height: {}", height_a, height_b);
    assert_eq!(height_a, 1, "Node A should be at height 1");
    assert_eq!(height_b, 2, "Node B should be at height 2");

    let block_a_str = get_rpc(8380, "/block/1").await.unwrap();
    let block_b_str = get_rpc(8381, "/block/1").await.unwrap();
    let block_a: serde_json::Value = serde_json::from_str(&block_a_str).unwrap();
    let block_b: serde_json::Value = serde_json::from_str(&block_b_str).unwrap();
    assert_ne!(block_a["hash"].as_str().unwrap(), block_b["hash"].as_str().unwrap(), "Height 1 block hashes must differ");

    // 3. Reconnect: Tell Node B to connect to Node A
    println!("Reconnecting nodes via RPC POST /peer...");
    let connect_payload = serde_json::json!({
        "addr": "127.0.0.1:9300"
    });
    let connect_response = post_rpc(8381, "/peer", &connect_payload.to_string()).await.unwrap();
    println!("Connect response status: {}", connect_response);

    // Wait for P2P connection, handshake, sync, and reorg to complete
    println!("Waiting for sync and reorganization...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    // 4. Verify canonical consensus tip has aligned on Node B's chain (Height 2)
    println!("Querying Node A and Node B tip state post-reconnect...");
    let final_status_a_str = get_rpc(8380, "/status").await.unwrap();
    let final_status_b_str = get_rpc(8381, "/status").await.unwrap();
    let final_status_a: serde_json::Value = serde_json::from_str(&final_status_a_str).unwrap();
    let final_status_b: serde_json::Value = serde_json::from_str(&final_status_b_str).unwrap();

    let final_height_a = final_status_a["height"].as_u64().unwrap();
    let final_height_b = final_status_b["height"].as_u64().unwrap();
    println!("Post-reconnect -> Node A Height: {}, Node B Height: {}", final_height_a, final_height_b);
    assert_eq!(final_height_a, 2, "Node A must have reorganized to height 2");
    assert_eq!(final_height_b, 2, "Node B must remain at height 2");

    let final_block_a_str = get_rpc(8380, "/block/2").await.unwrap();
    let final_block_b_str = get_rpc(8381, "/block/2").await.unwrap();
    let final_block_a: serde_json::Value = serde_json::from_str(&final_block_a_str).unwrap();
    let final_block_b: serde_json::Value = serde_json::from_str(&final_block_b_str).unwrap();
    assert_eq!(final_block_a["hash"].as_str().unwrap(), final_block_b["hash"].as_str().unwrap(), "Tip block hashes must align");
    assert_eq!(final_block_a["header"]["state_root"], final_block_b["header"]["state_root"], "State roots must be identical");

    // Verify Node A rolled back Branch A transaction, and applied Branch B transactions
    let sender_bech32 = sender.to_bech32m("sum").unwrap();
    let recipient_a_bech32 = recipient_a.to_bech32m("sum").unwrap();
    let recipient_b_bech32 = recipient_b.to_bech32m("sum").unwrap();

    let bal_sender: serde_json::Value = serde_json::from_str(&get_rpc(8380, &format!("/address/{}", sender_bech32)).await.unwrap()).unwrap();
    let bal_a: serde_json::Value = serde_json::from_str(&get_rpc(8380, &format!("/address/{}", recipient_a_bech32)).await.unwrap()).unwrap();
    let bal_b: serde_json::Value = serde_json::from_str(&get_rpc(8380, &format!("/address/{}", recipient_b_bech32)).await.unwrap()).unwrap();

    println!("Final Sender Balance: {}", bal_sender["balance"].as_u64().unwrap());
    println!("Final Recipient A (rolled back) Balance: {}", bal_a["balance"].as_u64().unwrap());
    println!("Final Recipient B (canonical) Balance: {}", bal_b["balance"].as_u64().unwrap());

    assert_eq!(bal_a["balance"].as_u64().unwrap(), 0, "Recipient A balance must be rolled back to 0");
    assert_eq!(bal_b["balance"].as_u64().unwrap(), 500_000, "Recipient B balance must reflect Branch B transactions");

    rpc_handle_a.abort();
    rpc_handle_b.abort();
    println!("E2E Network Fork Choice and Reorganization simulation test passed!");
}
