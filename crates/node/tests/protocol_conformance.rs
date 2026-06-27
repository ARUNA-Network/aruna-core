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

#[test]
fn test_conformance_100_transactions_identical_state() {
    let path_c = temp_db_path("node_c");
    let path_d = temp_db_path("node_d");
    let _cleaner_c = TempDirCleaner { path: path_c.clone() };
    let _cleaner_d = TempDirCleaner { path: path_d.clone() };

    let keypair = Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let pkh = derive_pubkey_hash(&pubkey);
    let sender = Address::from_pubkey_hash(pkh);

    let (storage_c, _state_c, engine_c, genesis_hash) = initialize_node_state(&path_c, &sender);
    let (storage_d, _state_d, engine_d, _) = initialize_node_state(&path_d, &sender);

    // 1. Generate 100 deterministic transactions
    let mut transactions = Vec::with_capacity(100);
    for nonce_val in 1..=100 {
        // Shift offset to avoid overlap with consensus reward accounts
        let recipient = Address::from_pubkey_hash([nonce_val as u8 + 100; 20]);
        let payload = TransactionPayload {
            nonce: Nonce(nonce_val),
            sender,
            recipient,
            amount: 1000,
            fee: 10,
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

    // 2. Group into 5 blocks (20 transactions per block) and commit on Node C
    let block_count = 5;
    let tx_per_block = 20;
    let mut committed_blocks = Vec::with_capacity(block_count);
    let mut parent_hash = genesis_hash;
    let mut parent_state_root = Hash::zero();

    for b_idx in 0..block_count {
        let block_txs = transactions[(b_idx * tx_per_block)..((b_idx + 1) * tx_per_block)].to_vec();
        let body = BlockBody {
            transactions: block_txs,
            validator_metadata: vec![],
            ecosystem_metadata: vec![],
        };
        let merkle_root = ConsensusEngine::calculate_merkle_root(&body.transactions).unwrap();
        let mut header = BlockHeader {
            version: 1,
            prev_block_hash: parent_hash,
            merkle_root,
            state_root: Hash::zero(),
            timestamp: 1782252030 + (b_idx as u64 * 30),
            difficulty: Difficulty(504381424),
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        };

        let temp_block = Block { header: header.clone(), body: body.clone() };
        let state_root = engine_c.calculate_state_root(parent_state_root, &temp_block).unwrap();
        header.state_root = state_root;

        let block = Block { header, body };
        let block_hash = engine_c.commit_block(&block).unwrap();

        parent_hash = block_hash;
        parent_state_root = state_root;
        committed_blocks.push(block);
    }

    // 3. Commit identical blocks on Node D
    for block in &committed_blocks {
        let _ = engine_d.commit_block(block).unwrap();
    }

    // 4. Assertions for exact determinism
    assert_eq!(storage_c.get_chain_height().unwrap().unwrap(), 5);
    assert_eq!(storage_d.get_chain_height().unwrap().unwrap(), 5);

    let tip_hash_c = storage_c.get_best_block().unwrap().unwrap();
    let tip_hash_d = storage_d.get_best_block().unwrap().unwrap();
    assert_eq!(tip_hash_c, tip_hash_d);

    let header_c = storage_c.get_block_header(&tip_hash_c).unwrap().unwrap();
    let header_d = storage_d.get_block_header(&tip_hash_d).unwrap().unwrap();
    assert_eq!(header_c.state_root, header_d.state_root);

    // Verify Sender Account state
    let (bal_sender_c, nonce_sender_c, _, _) = storage_c.get_account(&sender).unwrap().unwrap();
    let (bal_sender_d, nonce_sender_d, _, _) = storage_d.get_account(&sender).unwrap().unwrap();
    assert_eq!(bal_sender_c, bal_sender_d);
    assert_eq!(nonce_sender_c, nonce_sender_d);
    assert_eq!(nonce_sender_c, 100);

    // Verify all 100 Recipient Account states
    for nonce_val in 1..=100 {
        let recipient = Address::from_pubkey_hash([nonce_val as u8 + 100; 20]);
        let (bal_rec_c, nonce_rec_c, _, _) = storage_c.get_account(&recipient).unwrap().unwrap();
        let (bal_rec_d, nonce_rec_d, _, _) = storage_d.get_account(&recipient).unwrap().unwrap();
        assert_eq!(bal_rec_c, bal_rec_d);
        assert_eq!(nonce_rec_c, nonce_rec_d);
        assert_eq!(bal_rec_c, 1000);
    }

    println!("Protocol Conformance Test for 100 transactions successful! Node C and Node D state roots match exactly.");
}

#[test]
fn test_fork_handling_reconnect_reorg() {
    let path_a = temp_db_path("fork_a");
    let path_b = temp_db_path("fork_b");
    let _cleaner_a = TempDirCleaner { path: path_a.clone() };
    let _cleaner_b = TempDirCleaner { path: path_b.clone() };

    // Set up two keypairs and two different addresses
    let keypair_a = Ed25519Keypair::generate();
    let pubkey_a = keypair_a.public_key_bytes();
    let sender_a = Address::from_pubkey_hash(derive_pubkey_hash(&pubkey_a));

    let keypair_b = Ed25519Keypair::generate();
    let pubkey_b = keypair_b.public_key_bytes();
    let sender_b = Address::from_pubkey_hash(derive_pubkey_hash(&pubkey_b));

    let recipient_a = Address::from_pubkey_hash([0xaa; 20]);
    let recipient_b = Address::from_pubkey_hash([0xbb; 20]);

    // Setup helper that funds BOTH sender_a and sender_b on genesis state
    let setup_fork_db = |path: &std::path::Path| -> (Storage, StateManager, ConsensusEngine, Hash) {
        let storage = Storage::open(path).expect("Failed to open storage");
        let state_manager = StateManager::new(storage.clone());
        let engine = ConsensusEngine::new(state_manager.clone(), storage.clone());

        let mut init_batch = StorageBatch::new();
        init_batch.put_account(&sender_a, 1_000_000, 0, &Hash::zero(), &Hash::zero());
        init_batch.put_account(&sender_b, 1_000_000, 0, &Hash::zero(), &Hash::zero());
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
        let genesis_body = BlockBody { transactions: vec![], validator_metadata: vec![], ecosystem_metadata: vec![] };
        let genesis_bytes = serialize(&genesis_header).unwrap();
        let genesis_hash = aruna_crypto::blake3_hash(&genesis_bytes);

        storage.put_block_header(&genesis_hash, &genesis_header).unwrap();
        storage.put_block_body(&genesis_hash, &genesis_body).unwrap();
        storage.put_best_block(&genesis_hash).unwrap();
        storage.put_chain_height(0).unwrap();
        storage.put_cumulative_difficulty(&genesis_hash, 0).unwrap();
        storage.put_block_height_by_hash(&genesis_hash, 0).unwrap();

        (storage, state_manager, engine, genesis_hash)
    };

    let (storage_a, _state_a, engine_a, genesis_hash) = setup_fork_db(&path_a);
    let (_storage_b, _state_b, engine_b, _) = setup_fork_db(&path_b);

    // 1. Build Branch A on Node A: 10 blocks (1A..10A)
    // 1. Build Branch A on Node A: 10 blocks (1A..10A)
    let mut parent_hash_a = genesis_hash;
    let mut parent_state_a = Hash::zero();
    let mut branch_a_hashes = Vec::with_capacity(10);
    for i in 1..=10 {
        let payload = TransactionPayload {
            nonce: Nonce(i),
            sender: sender_a,
            recipient: recipient_a,
            amount: 100,
            fee: 10,
            gas_limit: 0,
            gas_price: 0,
            data: vec![],
        };
        let sig = keypair_a.sign(&serialize(&payload).unwrap()).to_vec();
        let tx = TransactionEnvelope {
            payload,
            signature_type: SignatureType::Ed25519,
            signature: sig,
            public_key: pubkey_a.to_vec(),
        };
        let body = BlockBody { transactions: vec![tx], validator_metadata: vec![], ecosystem_metadata: vec![] };
        let merkle_root = ConsensusEngine::calculate_merkle_root(&body.transactions).unwrap();
        let mut header = BlockHeader {
            version: 1,
            prev_block_hash: parent_hash_a,
            merkle_root,
            state_root: Hash::zero(),
            timestamp: 1782252030 + (i as u64 * 30),
            difficulty: Difficulty(504381424),
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        };

        let temp = Block { header: header.clone(), body: body.clone() };
        let state_root = engine_a.calculate_state_root(parent_state_a, &temp).unwrap();
        header.state_root = state_root;

        let block = Block { header, body };
        parent_hash_a = engine_a.commit_block(&block).unwrap();
        parent_state_a = state_root;
        branch_a_hashes.push(parent_hash_a);
    }

    assert_eq!(storage_a.get_chain_height().unwrap().unwrap(), 10);

    // 2. Build Branch B on Node B: 11 blocks (1B..11B)
    let mut parent_hash_b = genesis_hash;
    let mut parent_state_b = Hash::zero();
    let mut branch_b_blocks = Vec::with_capacity(11);
    for i in 1..=11 {
        let payload = TransactionPayload {
            nonce: Nonce(i),
            sender: sender_b,
            recipient: recipient_b,
            amount: 200,
            fee: 10,
            gas_limit: 0,
            gas_price: 0,
            data: vec![],
        };
        let sig = keypair_b.sign(&serialize(&payload).unwrap()).to_vec();
        let tx = TransactionEnvelope {
            payload,
            signature_type: SignatureType::Ed25519,
            signature: sig,
            public_key: pubkey_b.to_vec(),
        };
        let body = BlockBody { transactions: vec![tx], validator_metadata: vec![], ecosystem_metadata: vec![] };
        let merkle_root = ConsensusEngine::calculate_merkle_root(&body.transactions).unwrap();
        let mut header = BlockHeader {
            version: 1,
            prev_block_hash: parent_hash_b,
            merkle_root,
            state_root: Hash::zero(),
            timestamp: 1782252035 + (i as u64 * 30),
            difficulty: Difficulty(504381424),
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        };

        let temp = Block { header: header.clone(), body: body.clone() };
        let state_root = engine_b.calculate_state_root(parent_state_b, &temp).unwrap();
        header.state_root = state_root;

        let block = Block { header, body };
        parent_hash_b = engine_b.commit_block(&block).unwrap();
        parent_state_b = state_root;
        branch_b_blocks.push(block);
    }

    // 3. Connect (Feed Branch B blocks to Node A)
    // Blocks 1B to 10B should be stored as side-chain blocks (cumulative diff <= 10)
    for idx in 0..10 {
        let _h = engine_a.commit_block(&branch_b_blocks[idx]).unwrap();
        assert_eq!(storage_a.get_best_block().unwrap().unwrap(), parent_hash_a); // Node A tip remains 10A
        assert_eq!(storage_a.get_chain_height().unwrap().unwrap(), 10);

        // Assert that the canonical height-to-hash mapping is still Branch A hashes!
        let canonical_hash = storage_a.get_block_hash_by_height(idx as u64 + 1).unwrap().unwrap();
        assert_eq!(canonical_hash, branch_a_hashes[idx]);
    }

    // Committing 11B should trigger deep chain reorg!
    let hash_11b = engine_a.commit_block(&branch_b_blocks[10]).unwrap();
    assert_eq!(hash_11b, parent_hash_b);

    // 4. Assert Node A successfully switches canonical tip to 11B
    assert_eq!(storage_a.get_best_block().unwrap().unwrap(), parent_hash_b);
    assert_eq!(storage_a.get_chain_height().unwrap().unwrap(), 11);

    // Sender A and Recipient A should be rolled back to initial state (1,000,000 and 0)
    let (bal_sender_a, nonce_sender_a, _, _) = storage_a.get_account(&sender_a).unwrap().unwrap_or((0, 0, Hash::zero(), Hash::zero()));
    let (bal_rec_a, _, _, _) = storage_a.get_account(&recipient_a).unwrap().unwrap_or((0, 0, Hash::zero(), Hash::zero()));
    assert_eq!(bal_sender_a, 1_000_000);
    assert_eq!(nonce_sender_a, 0);
    assert_eq!(bal_rec_a, 0);

    // Sender B and Recipient B should be successfully applied (11 transfers of 200)
    let (bal_sender_b, nonce_sender_b, _, _) = storage_a.get_account(&sender_b).unwrap().unwrap();
    let (bal_rec_b, _, _, _) = storage_a.get_account(&recipient_b).unwrap().unwrap();
    // 1,000,000 - (11 * (200 + 10)) = 1,000,000 - 2310 = 997,690
    assert_eq!(bal_sender_b, 997690);
    assert_eq!(nonce_sender_b, 11);
    assert_eq!(bal_rec_b, 2200);

    println!("Fork choice deep reorganization test successful! Reconnect and reorg resolved correctly.");
}
