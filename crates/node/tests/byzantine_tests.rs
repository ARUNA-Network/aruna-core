//! End-to-end Byzantine and Invalid Messages integration test suite.
//!
//! Asserts that:
//! 1. Transactions with invalid signatures are rejected over RPC.
//! 2. Replaying committed transactions (same nonce/signature) is rejected.
//! 3. Sending malformed P2P packets disconnects the peer gracefully without node crashes.
//! 4. Sending huge P2P packets (> 4 MB) disconnects the peer immediately without OOM.
//! 5. Committing duplicate blocks is handled gracefully (ignored early).

use aruna_primitives::{BlockBody, BlockHeader, Hash, Difficulty, Address, Nonce, TransactionPayload, TransactionEnvelope, SignatureType};
use aruna_storage::{Storage, StorageBatch};
use aruna_node::runtime::{NodeContext, rpc_loop, network_loop};
use aruna_primitives::serialize;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

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
    path.push(format!("aruna_byzantine_{}_{}", suffix, nanos));
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

async fn post_rpc(rpc_port: u16, path: &str, body: &str) -> Result<String, Box<dyn std::error::Error>> {
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

fn extract_body(response: &str) -> String {
    response.find("\r\n\r\n")
        .map(|i| response[i + 4..].to_string())
        .unwrap_or_else(|| response.to_string())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_byzantine_and_invalid_messages() {
    let path = temp_db_path("byz");
    let _cleaner = TempDirCleaner { path: path.clone() };

    let keypair = aruna_crypto::Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let pkh = aruna_crypto::derive_pubkey_hash(&pubkey);
    let sender = Address::from_pubkey_hash(pkh);

    let (storage, _genesis_hash) = initialize_genesis_state(&path, &sender);

    let context = Arc::new(NodeContext::new(
        storage.clone(),
        9400, // p2p port
        8480, // rpc port
        7777, // chain id
        path.clone(),
        3600, // manual mining
    ));

    // Start P2P and RPC servers
    network_loop::start_p2p(context.clone(), None).await;
    let rpc_ctx = context.clone();
    let rpc_handle = tokio::spawn(async move {
        let _ = rpc_loop::start_rpc_server(rpc_ctx).await;
    });

    // Give servers a moment to bind
    tokio::time::sleep(Duration::from_millis(500)).await;

    // ── 1. Invalid Signature Test ───────────────────────────────────────────
    println!("Testing Invalid Signature rejection...");
    let recipient = Address::from_pubkey_hash([0x09; 20]);
    let payload = TransactionPayload {
        nonce: Nonce(1),
        sender,
        recipient,
        amount: 50_000,
        fee: 5000,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let payload_bytes = serialize(&payload).unwrap();
    // Generate a valid signature, then mutate it to make it invalid
    let mut sig = keypair.sign(&payload_bytes).to_vec();
    if !sig.is_empty() {
        sig[0] ^= 0xFF; // corrupt the signature
    }
    let tx_invalid = TransactionEnvelope {
        payload,
        signature_type: SignatureType::Ed25519,
        signature: sig,
        public_key: pubkey.to_vec(),
    };
    let res_invalid = post_rpc(8480, "/tx", &serde_json::to_string(&tx_invalid).unwrap()).await.unwrap();
    println!("Invalid Signature Response: {}", res_invalid);
    assert!(res_invalid.contains("\"status\":\"error\""));
    assert!(res_invalid.contains("InvalidSignature") || res_invalid.contains("verification failed"));

    // ── 2. Replay Transaction Test ──────────────────────────────────────────
    println!("Testing Replay Transaction rejection...");
    // Create a valid transaction with nonce 1
    let payload_valid = TransactionPayload {
        nonce: Nonce(1),
        sender,
        recipient,
        amount: 50_000,
        fee: 5000,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let sig_valid = keypair.sign(&serialize(&payload_valid).unwrap()).to_vec();
    let tx_valid = TransactionEnvelope {
        payload: payload_valid,
        signature_type: SignatureType::Ed25519,
        signature: sig_valid,
        public_key: pubkey.to_vec(),
    };
    let tx_valid_json = serde_json::to_string(&tx_valid).unwrap();

    // Submit it first time -> should succeed
    let res_valid1 = post_rpc(8480, "/tx", &tx_valid_json).await.unwrap();
    assert!(res_valid1.contains("\"status\":\"success\""));

    // Mine it into Block 1
    let txs = context.mempool.get_pending_transactions(100);
    let block1 = context.consensus_engine.produce_block(txs).unwrap();
    let hash1 = context.consensus_engine.commit_block(&block1).unwrap();
    assert_eq!(context.storage.get_chain_height().unwrap().unwrap(), 1);

    // Clear mempool
    let committed_hashes = vec![aruna_crypto::blake3_hash(&serialize(&tx_valid).unwrap())];
    context.mempool.remove_transactions(&committed_hashes);

    // Replay the exact same transaction again (nonce is now too low / already spent)
    let res_replay = post_rpc(8480, "/tx", &tx_valid_json).await.unwrap();
    println!("Replay Transaction Response: {}", res_replay);
    assert!(res_replay.contains("too low") || res_replay.contains("DuplicateNonce"));

    // ── 3. Malformed Packet Test ────────────────────────────────────────────
    println!("Testing Malformed P2P Packet handling...");
    {
        let mut stream = tokio::net::TcpStream::connect("127.0.0.1:9400").await.unwrap();
        // Write random garbage bytes (malformed packet)
        stream.write_all(&[0xDE, 0xAD, 0xBE, 0xEF, 0x12, 0x34, 0x56, 0x78]).await.unwrap();
        
        // Wait and check if connection gets dropped by node (read_to_end completes due to EOF)
        let mut data = Vec::new();
        let read_res = tokio::time::timeout(Duration::from_millis(500), stream.read_to_end(&mut data)).await;
        println!("Malformed packet check: completed with {:?}", read_res);
        assert!(read_res.is_ok(), "Node must disconnect peer and close the connection (EOF)");
    }

    // ── 4. Huge Packet Test ─────────────────────────────────────────────────
    println!("Testing Huge Packet DDoS rejection...");
    {
        let mut stream = tokio::net::TcpStream::connect("127.0.0.1:9400").await.unwrap();
        // Send a length prefix of 20 MB (20 * 1024 * 1024)
        let huge_len = (20 * 1024 * 1024_u32).to_be_bytes();
        stream.write_all(&huge_len).await.unwrap();
        
        // Wait and check if connection gets dropped immediately without crash
        let mut data = Vec::new();
        let read_res = tokio::time::timeout(Duration::from_millis(500), stream.read_to_end(&mut data)).await;
        println!("Huge packet check: completed with {:?}", read_res);
        assert!(read_res.is_ok(), "Node must drop connection immediately on huge packet length prefix");
    }

    // ── 5. Duplicate Block Test ─────────────────────────────────────────────
    println!("Testing Duplicate Block handling...");
    // Re-commit Block 1 (which is already committed)
    let re_commit_res = context.consensus_engine.commit_block(&block1);
    println!("Re-commit duplicate block result: {:?}", re_commit_res);
    // Should return Ok(hash) and not error or panic
    assert!(re_commit_res.is_ok());
    assert_eq!(re_commit_res.unwrap(), hash1);
    // Height should still be 1
    assert_eq!(context.storage.get_chain_height().unwrap().unwrap(), 1);

    rpc_handle.abort();
    println!("All Byzantine / Invalid Message tests passed successfully!");
}
