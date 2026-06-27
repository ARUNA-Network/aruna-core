//! Protocol conformance test suite for the ARUNA Network ledger state.
//! Verifies that two independent nodes produce identical state roots and balances
//! after processing the same sequence of transaction executions.

use aruna_primitives::{Block, BlockBody, BlockHeader, Hash, Difficulty, Address, Nonce, TransactionPayload, TransactionEnvelope, SignatureType};
use aruna_storage::{Storage, StorageBatch};
use aruna_state::StateManager;
use aruna_consensus::ConsensusEngine;
use aruna_crypto::{Ed25519Keypair, derive_pubkey_hash};
use aruna_primitives::serialize;
use std::path::PathBuf;
use std::time::SystemTime;

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
    path.push(format!("aruna_conformance_{}_{}", suffix, nanos));
    path
}

fn initialize_node_state(path: &std::path::Path, sender: &Address) -> (Storage, StateManager, ConsensusEngine, Hash) {
    let storage = Storage::open(path).expect("Failed to open storage");
    let state_manager = StateManager::new(storage.clone());
    let engine = ConsensusEngine::new(state_manager.clone(), storage.clone());

    // Pre-fund sender in genesis state
    let mut init_batch = StorageBatch::new();
    init_batch.put_account(sender, 100_000_000, 0, &Hash::zero(), &Hash::zero());
    storage.write_batch(init_batch).unwrap();

    // Setup Genesis block (height 0)
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

    (storage, state_manager, engine, genesis_hash)
}

#[test]
fn test_conformance_two_nodes_identical_state() {
    let path_a = temp_db_path("node_a");
    let path_b = temp_db_path("node_b");
    let _cleaner_a = TempDirCleaner { path: path_a.clone() };
    let _cleaner_b = TempDirCleaner { path: path_b.clone() };

    let keypair = Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let pkh = derive_pubkey_hash(&pubkey);
    let sender = Address::from_pubkey_hash(pkh);

    let (storage_a, _state_a, engine_a, genesis_hash) = initialize_node_state(&path_a, &sender);
    let (storage_b, _state_b, engine_b, _) = initialize_node_state(&path_b, &sender);

    // Generate 10 test transactions sending to multiple recipient addresses
    let mut transactions = Vec::new();
    for nonce_val in 1..=10 {
        let recipient = Address::from_pubkey_hash([nonce_val as u8 + 100; 20]);
        let payload = TransactionPayload {
            nonce: Nonce(nonce_val),
            sender,
            recipient,
            amount: 10_000 * nonce_val, // varying amounts
            fee: 100,
            gas_limit: 0,
            gas_price: 0,
            data: vec![],
        };
        let sig = keypair.sign(&serialize(&payload).unwrap()).to_vec();
        let tx = TransactionEnvelope {
            payload,
            signature_type: SignatureType::Ed25519,
            signature: sig,
            public_key: pubkey.to_vec(),
        };
        transactions.push(tx);
    }

    // Group transactions into 2 blocks: Block 1 (txs 0-4), Block 2 (txs 5-9)
    let block_1_txs = transactions[0..5].to_vec();
    let block_2_txs = transactions[5..10].to_vec();

    // Node A produces and commits Block 1
    let block_1_body = BlockBody {
        transactions: block_1_txs,
        validator_metadata: vec![],
        ecosystem_metadata: vec![],
    };
    let block_1_merkle = ConsensusEngine::calculate_merkle_root(&block_1_body.transactions).unwrap();
    let mut block_1_header = BlockHeader {
        version: 1,
        prev_block_hash: genesis_hash,
        merkle_root: block_1_merkle,
        state_root: Hash::zero(),
        timestamp: 1782252030,
        difficulty: Difficulty(504381424),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let block_1_temp = Block { header: block_1_header, body: block_1_body };
    let state_root_1 = engine_a.calculate_state_root(Hash::zero(), &block_1_temp).unwrap();
    block_1_header.state_root = state_root_1;
    let block_1 = Block { header: block_1_header, body: block_1_temp.body };
    let hash_1 = engine_a.commit_block(&block_1).unwrap();

    // Node A produces and commits Block 2
    let block_2_body = BlockBody {
        transactions: block_2_txs,
        validator_metadata: vec![],
        ecosystem_metadata: vec![],
    };
    let block_2_merkle = ConsensusEngine::calculate_merkle_root(&block_2_body.transactions).unwrap();
    let mut block_2_header = BlockHeader {
        version: 1,
        prev_block_hash: hash_1,
        merkle_root: block_2_merkle,
        state_root: Hash::zero(),
        timestamp: 1782252060,
        difficulty: Difficulty(504381424),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let block_2_temp = Block { header: block_2_header, body: block_2_body };
    let state_root_2 = engine_a.calculate_state_root(state_root_1, &block_2_temp).unwrap();
    block_2_header.state_root = state_root_2;
    let block_2 = Block { header: block_2_header, body: block_2_temp.body };
    let hash_2 = engine_a.commit_block(&block_2).unwrap();

    // Node B receives and commits the exact same blocks
    let hash_1_b = engine_b.commit_block(&block_1).unwrap();
    let hash_2_b = engine_b.commit_block(&block_2).unwrap();

    // Asserts: Block hashes must match exactly
    assert_eq!(hash_1, hash_1_b);
    assert_eq!(hash_2, hash_2_b);

    // Tip heights must match
    assert_eq!(storage_a.get_chain_height().unwrap().unwrap(), 2);
    assert_eq!(storage_b.get_chain_height().unwrap().unwrap(), 2);

    // Tip block hashes must match
    assert_eq!(storage_a.get_best_block().unwrap().unwrap(), hash_2);
    assert_eq!(storage_b.get_best_block().unwrap().unwrap(), hash_2);

    // Crucial Check: State root commitment in committed headers must match
    let header_a = storage_a.get_block_header(&hash_2).unwrap().unwrap();
    let header_b = storage_b.get_block_header(&hash_2).unwrap().unwrap();
    assert_eq!(header_a.state_root, header_b.state_root);
    assert_ne!(header_a.state_root, Hash::zero());

    // Verify individual account state roundtrips are identical on both nodes
    let (bal_sender_a, nonce_sender_a, _, _) = storage_a.get_account(&sender).unwrap().unwrap();
    let (bal_sender_b, nonce_sender_b, _, _) = storage_b.get_account(&sender).unwrap().unwrap();
    assert_eq!(bal_sender_a, bal_sender_b);
    assert_eq!(nonce_sender_a, nonce_sender_b);

    for nonce_val in 1..=10 {
        let recipient = Address::from_pubkey_hash([nonce_val as u8 + 100; 20]);
        let (bal_rec_a, nonce_rec_a, _, _) = storage_a.get_account(&recipient).unwrap().unwrap();
        let (bal_rec_b, nonce_rec_b, _, _) = storage_b.get_account(&recipient).unwrap().unwrap();
        assert_eq!(bal_rec_a, bal_rec_b);
        assert_eq!(nonce_rec_a, nonce_rec_b);
        assert_eq!(bal_rec_a, 10_000 * nonce_val);
    }

    println!("Protocol Conformance Test successful! Node A and Node B state roots and accounts are identical.");
}
