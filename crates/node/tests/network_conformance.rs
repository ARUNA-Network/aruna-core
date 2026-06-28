//! End-to-end network-level determinism integration test.
//!
//! Node A is started on P2P port 9200, RPC port 8280 with a 1-second block time.
//! Node B is started on P2P port 9201, RPC port 8281 connected to Node A.
//! We broadcast 100 transactions to Node A's RPC, wait for blocks to be mined,
//! propagated, and verified on Node B, and then assert that both nodes achieve
//! identical heights, tip block hashes, and state roots over their RPC interfaces.

use aruna_primitives::{BlockBody, BlockHeader, Hash, Difficulty, Address, Nonce, TransactionPayload, TransactionEnvelope, SignatureType};
use aruna_storage::{Storage, StorageBatch};
use aruna_node::runtime::{NodeContext, rpc_loop, network_loop, block_loop};
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
    path.push(format!("aruna_network_conformance_{}_{}", suffix, nanos));
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

// ── Test Execution ────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_network_determinism_100_txs() {
    let path_a = temp_db_path("node_a");
    let path_b = temp_db_path("node_b");
    let _cleaner_a = TempDirCleaner { path: path_a.clone() };
    let _cleaner_b = TempDirCleaner { path: path_b.clone() };

    let keypair = aruna_crypto::Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let pkh = aruna_crypto::derive_pubkey_hash(&pubkey);
    let sender = Address::from_pubkey_hash(pkh);

    let (storage_a, _) = initialize_genesis_state(&path_a, &sender);
    let (storage_b, _) = initialize_genesis_state(&path_b, &sender);

    // Node A: 1-second block time for fast test execution
    let context_a = Arc::new(NodeContext::new(
        storage_a,
        9200, // p2p port
        8280, // rpc port
        7777, // chain id
        path_a.clone(),
        1,    // 1-second block time
    ));

    // Node B: 3600-second block time (does not mine, only receives via P2P gossip)
    let context_b = Arc::new(NodeContext::new(
        storage_b,
        9201, // p2p port
        8281, // rpc port
        7777, // chain id
        path_b.clone(),
        3600, // 1-hour block time
    ));

    // Start Node A loops
    network_loop::start_p2p(context_a.clone(), None).await;
    block_loop::start_block_producer(context_a.clone());
    let rpc_ctx_a = context_a.clone();
    let rpc_handle_a = tokio::spawn(async move {
        let _ = rpc_loop::start_rpc_server(rpc_ctx_a).await;
    });

    // Start Node B loops (connecting to Node A)
    network_loop::start_p2p(context_b.clone(), Some("127.0.0.1:9200".parse().unwrap())).await;
    let rpc_ctx_b = context_b.clone();
    let rpc_handle_b = tokio::spawn(async move {
        let _ = rpc_loop::start_rpc_server(rpc_ctx_b).await;
    });

    // Wait a brief moment for handshake to complete and servers to bind
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // 1. Generate 100 valid transactions
    println!("Generating 100 conformance transactions...");
    let mut envelopes = Vec::new();
    for nonce_val in 1..=100 {
        let recipient = Address::from_pubkey_hash([nonce_val as u8 + 100; 20]);
        let payload = TransactionPayload {
            nonce: Nonce(nonce_val),
            sender,
            recipient,
            amount: 1000,
            fee: 5000,
            gas_limit: 0,
            gas_price: 0,
            data: vec![],
        };
        let payload_bytes = serialize(&payload).unwrap();
        let signature = keypair.sign(&payload_bytes).to_vec();
        let envelope = TransactionEnvelope {
            payload,
            signature_type: SignatureType::Ed25519,
            signature,
            public_key: pubkey.to_vec(),
        };
        envelopes.push(envelope);
    }

    // 2. Submit all 100 transactions to Node A via RPC
    println!("Submitting 100 transactions to Node A over HTTP RPC...");
    for envelope in envelopes {
        let tx_json = serde_json::to_string(&envelope).unwrap();
        let submit_response = post_rpc(8280, "/tx", &tx_json)
            .await
            .expect("Failed to broadcast tx to Node A");
        assert!(submit_response.contains("\"status\":\"success\""), "Transaction submit must succeed: {}", submit_response);
    }

    // 3. Wait for Node A to pack blocks and Node B to receive/commit them via P2P
    // At block time = 1s, it should mine within 1-2 seconds. We wait 4 seconds for safety.
    println!("Waiting for block production and propagation...");
    tokio::time::sleep(Duration::from_secs(4)).await;

    // 4. Query RPC status of both nodes to assert identical states
    println!("Querying Node A and Node B status...");
    let status_a_str = get_rpc(8280, "/status").await.expect("Failed to query status A");
    let status_b_str = get_rpc(8281, "/status").await.expect("Failed to query status B");

    let status_a: serde_json::Value = serde_json::from_str(&status_a_str).unwrap();
    let status_b: serde_json::Value = serde_json::from_str(&status_b_str).unwrap();

    let height_a = status_a["height"].as_u64().unwrap();
    let height_b = status_b["height"].as_u64().unwrap();

    println!("Node A Height: {}, Node B Height: {}", height_a, height_b);
    assert!(height_a > 0, "Node A should have produced at least 1 block");
    assert_eq!(height_a, height_b, "Node A and Node B heights must match");

    // Fetch block at tip height from both Node A and Node B RPCs
    let block_a_str = get_rpc(8280, &format!("/block/{}", height_a)).await.expect("Failed to query block A");
    let block_b_str = get_rpc(8281, &format!("/block/{}", height_b)).await.expect("Failed to query block B");

    let block_a: serde_json::Value = serde_json::from_str(&block_a_str).unwrap();
    let block_b: serde_json::Value = serde_json::from_str(&block_b_str).unwrap();

    let hash_a = block_a["hash"].as_str().unwrap();
    let hash_b = block_b["hash"].as_str().unwrap();
    assert_eq!(hash_a, hash_b, "Tip block hashes must match exactly");

    assert_eq!(block_a["header"]["state_root"], block_b["header"]["state_root"], "Tip state roots must match exactly");

    // Verify sender account balances match exactly via RPC
    let sender_bech32 = sender.to_bech32m("sum").unwrap();
    let balance_a_str = get_rpc(8280, &format!("/address/{}", sender_bech32)).await.expect("Failed to query balance A");
    let balance_b_str = get_rpc(8281, &format!("/address/{}", sender_bech32)).await.expect("Failed to query balance B");

    let bal_a: serde_json::Value = serde_json::from_str(&balance_a_str).unwrap();
    let bal_b: serde_json::Value = serde_json::from_str(&balance_b_str).unwrap();
    assert_eq!(bal_a["balance"].as_u64().unwrap(), bal_b["balance"].as_u64().unwrap(), "Sender balance must match exactly");

    rpc_handle_a.abort();
    rpc_handle_b.abort();
    println!("E2E Network Conformance and Determinism test passed successfully!");
}
