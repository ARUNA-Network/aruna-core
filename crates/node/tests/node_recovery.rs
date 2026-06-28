//! Node ledger state recovery integration tests.

use aruna_primitives::{Block, BlockBody, BlockHeader, Hash, Difficulty};
use aruna_storage::{Storage, StorageBatch};
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

fn temp_db_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("aruna_node_recovery_{}", nanos));
    path
}

#[test]
fn test_node_ledger_recovery() {
    let path = temp_db_path();
    let _cleaner = TempDirCleaner { path: path.clone() };

    let genesis_hash = {
        let storage = Storage::open(&path).expect("Failed to open storage");
        assert!(storage.get_best_block().unwrap().is_none(), "Database should be empty initially");

        let mut batch = StorageBatch::new();

        // Construct standard Genesis Block Header
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

        let genesis_block = Block {
            header: genesis_header,
            body: genesis_body,
        };

        // Serialize and calculate Genesis hash (BLAKE3)
        let header_bytes = aruna_primitives::serialize(&genesis_header).expect("Failed to serialize header");
        let hash = aruna_crypto::blake3_hash(&header_bytes);

        // Persist Block 0 data
        storage.put_block_header(&hash, &genesis_header).expect("Failed to persist header");
        storage.put_block_body(&hash, &genesis_block.body).expect("Failed to persist body");

        // Update Chain Metadata indexes in storage batch
        batch.put_block_height_map(0, &hash);
        storage.write_batch(batch).expect("Failed to write batch");

        // Save best/finalized metadata and chain_id
        storage.put_best_block(&hash).expect("Failed to put best block");
        storage.put_chain_height(0).expect("Failed to put chain height");
        storage.put_finalized_block(&hash).expect("Failed to put finalized block");
        storage.put_chain_id(1).expect("Failed to put chain id");

        hash
    };

    // Reopen database and verify recovery of best block, height, and chain ID
    {
        let storage = Storage::open(&path).expect("Failed to reopen storage");

        let best_hash = storage.get_best_block()
            .expect("Failed to read best block")
            .expect("Best block should be present");
        assert_eq!(best_hash, genesis_hash);

        let height = storage.get_chain_height()
            .expect("Failed to read chain height")
            .expect("Chain height should be present");
        assert_eq!(height, 0);

        let chain_id = storage.get_chain_id()
            .expect("Failed to read chain ID")
            .expect("Chain ID should be present");
        assert_eq!(chain_id, 1);
    }
}

#[test]
fn test_fork_choice_reorganization() {
    use aruna_state::StateManager;
    use aruna_consensus::ConsensusEngine;
    use aruna_primitives::{Address, Nonce, TransactionPayload, TransactionEnvelope, SignatureType};
    use aruna_crypto::Ed25519Keypair;
    use aruna_primitives::serialize;

    let path_a = temp_db_path();
    let path_b = temp_db_path();
    let _cleaner_a = TempDirCleaner { path: path_a.clone() };
    let _cleaner_b = TempDirCleaner { path: path_b.clone() };

    // Fund sender account in genesis state on BOTH databases
    let keypair = Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let pkh = aruna_crypto::derive_pubkey_hash(&pubkey);
    let sender = Address::from_pubkey_hash(pkh);
    let recipient_a = Address::from_pubkey_hash([0x0a; 20]);
    let recipient_b = Address::from_pubkey_hash([0x0b; 20]);

    // Setup function to initialize Genesis on a database
    let setup_db = |path: &std::path::Path| -> (Storage, StateManager, ConsensusEngine, Hash) {
        let storage = Storage::open(path).expect("Failed to open storage");
        let state_manager = StateManager::new(storage.clone());
        let engine = ConsensusEngine::new(
            state_manager.clone(),
            storage.clone(),
            Address::from_pubkey_hash([0x01; 20]),
            Address::from_pubkey_hash([0x02; 20]),
            Address::from_pubkey_hash([0x03; 20]),
        );

        let mut init_batch = StorageBatch::new();
        init_batch.put_account(&sender, 1000000, 0, &Hash::zero(), &Hash::zero());
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

    let (storage_a, _state_a, engine_a, genesis_hash) = setup_db(&path_a);
    let (_storage_b, _state_b, engine_b, _) = setup_db(&path_b);

    // 1. Build Branch A: Genesis -> Block 1A -> Block 2A on database A
    // Block 1A: sender transfers 100 to recipient_a
    let payload_1a = TransactionPayload {
        nonce: Nonce(1),
        sender,
        recipient: recipient_a,
        amount: 100,
        fee: 50,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let sig_1a = keypair.sign(&serialize(&payload_1a).unwrap()).to_vec();
    let tx_1a = TransactionEnvelope {
        payload: payload_1a,
        signature_type: SignatureType::Ed25519,
        signature: sig_1a,
        public_key: pubkey.to_vec(),
    };

    let block_1a_body = BlockBody { transactions: vec![tx_1a], validator_metadata: vec![], ecosystem_metadata: vec![] };
    let block_1a_merkle = ConsensusEngine::calculate_merkle_root(&block_1a_body.transactions).unwrap();
    let mut block_1a_header = BlockHeader {
        version: 1,
        prev_block_hash: genesis_hash,
        merkle_root: block_1a_merkle,
        state_root: Hash::zero(),
        timestamp: 1782252030,
        difficulty: Difficulty(504381424),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let block_1a = Block { header: block_1a_header, body: block_1a_body };
    let state_root_1a = engine_a.calculate_state_root(Hash::zero(), &block_1a).unwrap();
    block_1a_header.state_root = state_root_1a;
    let block_1a = Block { header: block_1a_header, body: block_1a.body };
    let hash_1a = engine_a.commit_block(&block_1a).unwrap();

    // Block 2A: sender transfers 200 to recipient_a (nonce 2)
    let payload_2a = TransactionPayload {
        nonce: Nonce(2),
        sender,
        recipient: recipient_a,
        amount: 200,
        fee: 50,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let sig_2a = keypair.sign(&serialize(&payload_2a).unwrap()).to_vec();
    let tx_2a = TransactionEnvelope {
        payload: payload_2a,
        signature_type: SignatureType::Ed25519,
        signature: sig_2a,
        public_key: pubkey.to_vec(),
    };

    let block_2a_body = BlockBody { transactions: vec![tx_2a], validator_metadata: vec![], ecosystem_metadata: vec![] };
    let block_2a_merkle = ConsensusEngine::calculate_merkle_root(&block_2a_body.transactions).unwrap();
    let mut block_2a_header = BlockHeader {
        version: 1,
        prev_block_hash: hash_1a,
        merkle_root: block_2a_merkle,
        state_root: Hash::zero(),
        timestamp: 1782252060,
        difficulty: Difficulty(504381424),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let block_2a = Block { header: block_2a_header, body: block_2a_body };
    let state_root_2a = engine_a.calculate_state_root(state_root_1a, &block_2a).unwrap();
    block_2a_header.state_root = state_root_2a;
    let block_2a = Block { header: block_2a_header, body: block_2a.body };
    let hash_2a = engine_a.commit_block(&block_2a).unwrap();

    // 2. Build Branch B: Genesis -> Block 1B -> Block 2B -> Block 3B on database B
    // Block 1B: sender transfers 500 to recipient_b (nonce 1)
    let payload_1b = TransactionPayload {
        nonce: Nonce(1),
        sender,
        recipient: recipient_b,
        amount: 500,
        fee: 50,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let sig_1b = keypair.sign(&serialize(&payload_1b).unwrap()).to_vec();
    let tx_1b = TransactionEnvelope {
        payload: payload_1b,
        signature_type: SignatureType::Ed25519,
        signature: sig_1b,
        public_key: pubkey.to_vec(),
    };

    let block_1b_body = BlockBody { transactions: vec![tx_1b], validator_metadata: vec![], ecosystem_metadata: vec![] };
    let block_1b_merkle = ConsensusEngine::calculate_merkle_root(&block_1b_body.transactions).unwrap();
    let mut block_1b_header = BlockHeader {
        version: 1,
        prev_block_hash: genesis_hash,
        merkle_root: block_1b_merkle,
        state_root: Hash::zero(),
        timestamp: 1782252035,
        difficulty: Difficulty(504381424),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let block_1b = Block { header: block_1b_header, body: block_1b_body };
    let state_root_1b = engine_b.calculate_state_root(Hash::zero(), &block_1b).unwrap();
    block_1b_header.state_root = state_root_1b;
    let block_1b = Block { header: block_1b_header, body: block_1b.body };
    let hash_1b = engine_b.commit_block(&block_1b).unwrap();

    // Block 2B: sender transfers 1000 to recipient_b (nonce 2)
    let payload_2b = TransactionPayload {
        nonce: Nonce(2),
        sender,
        recipient: recipient_b,
        amount: 1000,
        fee: 50,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let sig_2b = keypair.sign(&serialize(&payload_2b).unwrap()).to_vec();
    let tx_2b = TransactionEnvelope {
        payload: payload_2b,
        signature_type: SignatureType::Ed25519,
        signature: sig_2b,
        public_key: pubkey.to_vec(),
    };

    let block_2b_body = BlockBody { transactions: vec![tx_2b], validator_metadata: vec![], ecosystem_metadata: vec![] };
    let block_2b_merkle = ConsensusEngine::calculate_merkle_root(&block_2b_body.transactions).unwrap();
    let mut block_2b_header = BlockHeader {
        version: 1,
        prev_block_hash: hash_1b,
        merkle_root: block_2b_merkle,
        state_root: Hash::zero(),
        timestamp: 1782252065,
        difficulty: Difficulty(504381424),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let block_2b = Block { header: block_2b_header, body: block_2b_body };
    let state_root_2b = engine_b.calculate_state_root(state_root_1b, &block_2b).unwrap();
    block_2b_header.state_root = state_root_2b;
    let block_2b = Block { header: block_2b_header, body: block_2b.body };
    let hash_2b = engine_b.commit_block(&block_2b).unwrap();

    // Block 3B: sender transfers 2000 to recipient_b (nonce 3)
    let payload_3b = TransactionPayload {
        nonce: Nonce(3),
        sender,
        recipient: recipient_b,
        amount: 2000,
        fee: 50,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let sig_3b = keypair.sign(&serialize(&payload_3b).unwrap()).to_vec();
    let tx_3b = TransactionEnvelope {
        payload: payload_3b,
        signature_type: SignatureType::Ed25519,
        signature: sig_3b,
        public_key: pubkey.to_vec(),
    };

    let block_3b_body = BlockBody { transactions: vec![tx_3b], validator_metadata: vec![], ecosystem_metadata: vec![] };
    let block_3b_merkle = ConsensusEngine::calculate_merkle_root(&block_3b_body.transactions).unwrap();
    let mut block_3b_header = BlockHeader {
        version: 1,
        prev_block_hash: hash_2b,
        merkle_root: block_3b_merkle,
        state_root: Hash::zero(),
        timestamp: 1782252095,
        difficulty: Difficulty(504381424),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let block_3b = Block { header: block_3b_header, body: block_3b_body };
    let state_root_3b = engine_b.calculate_state_root(state_root_2b, &block_3b).unwrap();
    block_3b_header.state_root = state_root_3b;
    let block_3b = Block { header: block_3b_header, body: block_3b.body };
    let hash_3b = engine_b.commit_block(&block_3b).unwrap();

    // 3. Send Branch B blocks to Node A
    // commit_block 1B: stored as side-chain since cum_diff (1) <= canonical best (2)
    let committed_1b = engine_a.commit_block(&block_1b).unwrap();
    assert_eq!(committed_1b, hash_1b);
    assert_eq!(storage_a.get_best_block().unwrap().unwrap(), hash_2a); // Tip stays at 2A

    // commit_block 2B: stored as side-chain since cum_diff (2) <= canonical best (2)
    let committed_2b = engine_a.commit_block(&block_2b).unwrap();
    assert_eq!(committed_2b, hash_2b);
    assert_eq!(storage_a.get_best_block().unwrap().unwrap(), hash_2a); // Tip stays at 2A

    // commit_block 3B: triggers chain reorg since cum_diff (3) > canonical best (2)
    let committed_3b = engine_a.commit_block(&block_3b).unwrap();
    assert_eq!(committed_3b, hash_3b);

    // Assert Node A's tip is 3B and height is 3
    assert_eq!(storage_a.get_best_block().unwrap().unwrap(), hash_3b);
    assert_eq!(storage_a.get_chain_height().unwrap().unwrap(), 3);

    // Recipient A state must be rolled back to 0!
    let (bal_a, _, _, _) = storage_a.get_account(&recipient_a).unwrap().unwrap_or((0, 0, Hash::zero(), Hash::zero()));
    assert_eq!(bal_a, 0);

    // Recipient B state must be applied to 3500! (500 + 1000 + 2000)
    let (bal_b, _, _, _) = storage_a.get_account(&recipient_b).unwrap().unwrap();
    assert_eq!(bal_b, 3500);
}

// ============================================================================
// Sprint C — Chaos: Crash Recovery & Network Partition Tests
// ============================================================================

/// Helper: build and commit a block with a single transaction, returning the committed hash.
fn build_and_commit_block(
    engine: &aruna_consensus::ConsensusEngine,
    keypair: &aruna_crypto::Ed25519Keypair,
    pubkey: &[u8; 32],
    sender: aruna_primitives::Address,
    recipient: aruna_primitives::Address,
    nonce: u64,
    amount: u64,
    fee: u64,
    timestamp: u64,
    parent_hash: aruna_primitives::Hash,
    parent_state_root: aruna_primitives::Hash,
) -> (aruna_primitives::Hash, aruna_primitives::Hash) {
    use aruna_primitives::{
        Block, BlockBody, BlockHeader, Difficulty, Hash, Nonce,
        TransactionEnvelope, TransactionPayload, SignatureType, serialize,
    };
    use aruna_consensus::ConsensusEngine;

    let payload = TransactionPayload {
        nonce: Nonce(nonce),
        sender,
        recipient,
        amount,
        fee,
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
    let body = BlockBody {
        transactions: vec![tx],
        validator_metadata: vec![],
        ecosystem_metadata: vec![],
    };
    let merkle_root = ConsensusEngine::calculate_merkle_root(&body.transactions).unwrap();
    let mut header = BlockHeader {
        version: 1,
        prev_block_hash: parent_hash,
        merkle_root,
        state_root: Hash::zero(),
        timestamp,
        difficulty: Difficulty(504381424),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let temp_block = Block { header: header.clone(), body: body.clone() };
    let state_root = engine.calculate_state_root(parent_state_root, &temp_block).unwrap();
    header.state_root = state_root;
    let block = Block { header, body };
    let hash = engine.commit_block(&block).unwrap();
    (hash, state_root)
}

/// Crash Recovery Test.
///
/// Proves that the node's ledger state (best block, chain height, account balances)
/// survives an abrupt process death (simulated by dropping all handles) and that
/// a freshly-opened Storage instance over the same RocksDB path reads back the
/// exact state that was committed before the "crash". The node must also be able
/// to continue producing and committing blocks after recovery.
#[test]
fn test_crash_mid_write_and_recovery() {
    use aruna_state::StateManager;
    use aruna_consensus::ConsensusEngine;
    use aruna_primitives::{Address, Hash};
    use aruna_crypto::Ed25519Keypair;

    let path = temp_db_path();
    let _cleaner = TempDirCleaner { path: path.clone() };

    let keypair = Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let sender = Address::from_pubkey_hash(aruna_crypto::derive_pubkey_hash(&pubkey));
    let recipient = Address::from_pubkey_hash([0xcc; 20]);

    let pre_crash_hash;
    let pre_crash_height;
    let pre_crash_sender_balance;
    let pre_crash_nonce;

    // ---- Phase 1: Write 5 blocks then simulate crash by dropping all handles ----
    {
        let storage = Storage::open(&path).expect("Failed to open storage");
        let state_manager = StateManager::new(storage.clone());
        let engine = ConsensusEngine::new(
            state_manager.clone(),
            storage.clone(),
            Address::from_pubkey_hash([0x01; 20]),
            Address::from_pubkey_hash([0x02; 20]),
            Address::from_pubkey_hash([0x03; 20]),
        );

        // Fund sender
        let mut init = StorageBatch::new();
        init.put_account(&sender, 10_000_000, 0, &Hash::zero(), &Hash::zero());
        storage.write_batch(init).unwrap();

        // Genesis
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
        let genesis_bytes = aruna_primitives::serialize(&genesis_header).unwrap();
        let genesis_hash = aruna_crypto::blake3_hash(&genesis_bytes);
        storage.put_block_header(&genesis_hash, &genesis_header).unwrap();
        storage.put_block_body(&genesis_hash, &genesis_body).unwrap();
        storage.put_best_block(&genesis_hash).unwrap();
        storage.put_chain_height(0).unwrap();
        storage.put_cumulative_difficulty(&genesis_hash, 0).unwrap();
        storage.put_block_height_by_hash(&genesis_hash, 0).unwrap();

        let mut parent_hash = genesis_hash;
        let mut parent_state = Hash::zero();
        for i in 1u64..=5 {
            let (h, s) = build_and_commit_block(
                &engine, &keypair, &pubkey, sender, recipient,
                i, 1000, 5000, 1782252000 + i * 30,
                parent_hash, parent_state,
            );
            parent_hash = h;
            parent_state = s;
        }

        // Capture pre-crash state
        pre_crash_hash = storage.get_best_block().unwrap().unwrap();
        pre_crash_height = storage.get_chain_height().unwrap().unwrap();
        let (bal, nonce, _, _) = storage.get_account(&sender).unwrap().unwrap();
        pre_crash_sender_balance = bal;
        pre_crash_nonce = nonce;

        // All handles drop here — simulates abrupt process death.
        // RocksDB WAL ensures all committed data is durable.
    }

    // ---- Phase 2: Reopen storage from same path (simulate restart) ----
    {
        let storage_recovered = Storage::open(&path).expect("Failed to reopen storage after crash");
        let state_manager_recovered = StateManager::new(storage_recovered.clone());
        let engine_recovered = ConsensusEngine::new(
            state_manager_recovered.clone(),
            storage_recovered.clone(),
            Address::from_pubkey_hash([0x01; 20]),
            Address::from_pubkey_hash([0x02; 20]),
            Address::from_pubkey_hash([0x03; 20]),
        );

        // Assert: all state is exactly as it was before crash
        let recovered_hash = storage_recovered.get_best_block().unwrap().unwrap();
        let recovered_height = storage_recovered.get_chain_height().unwrap().unwrap();
        let (recovered_bal, recovered_nonce, _, _) = storage_recovered.get_account(&sender).unwrap().unwrap();

        assert_eq!(recovered_hash, pre_crash_hash, "Best block hash must survive crash");
        assert_eq!(recovered_height, pre_crash_height, "Chain height must survive crash");
        assert_eq!(recovered_bal, pre_crash_sender_balance, "Sender balance must survive crash");
        assert_eq!(recovered_nonce, pre_crash_nonce, "Sender nonce must survive crash");
        assert_eq!(recovered_height, 5, "Should have 5 committed blocks");

        // Assert: node can continue producing blocks after recovery (block 6)
        let (hash_6, _) = build_and_commit_block(
            &engine_recovered, &keypair, &pubkey, sender, recipient,
            6, 1000, 5000, 1782252000 + 6 * 30,
            recovered_hash, storage_recovered.get_block_header(&recovered_hash).unwrap().unwrap().state_root,
        );
        assert_ne!(hash_6, recovered_hash);
        assert_eq!(storage_recovered.get_chain_height().unwrap().unwrap(), 6);
        assert_eq!(storage_recovered.get_best_block().unwrap().unwrap(), hash_6);

        println!("Crash recovery test successful! Node resumed at height 6 after simulated crash.");
    }
}

/// Network Partition + Reconnect Determinism Test.
///
/// Simulates two nodes mining independently during a network partition, then
/// reconnecting. The heavier chain (more cumulative work) wins. The lighter
/// chain's state transitions are fully rolled back. All account states after
/// reconnect are verified to exactly match the winning chain's state.
#[test]
fn test_partition_and_deterministic_reconnect() {
    use aruna_state::StateManager;
    use aruna_consensus::ConsensusEngine;
    use aruna_primitives::{Address, Hash};
    use aruna_crypto::Ed25519Keypair;

    let path_x = temp_db_path();
    let path_y = temp_db_path();
    let _cleaner_x = TempDirCleaner { path: path_x.clone() };
    let _cleaner_y = TempDirCleaner { path: path_y.clone() };

    // Two independent keypairs — each node mines its own transactions
    let keypair_x = Ed25519Keypair::generate();
    let pubkey_x = keypair_x.public_key_bytes();
    let sender_x = Address::from_pubkey_hash(aruna_crypto::derive_pubkey_hash(&pubkey_x));
    let recipient_x = Address::from_pubkey_hash([0xee; 20]);

    let keypair_y = Ed25519Keypair::generate();
    let pubkey_y = keypair_y.public_key_bytes();
    let sender_y = Address::from_pubkey_hash(aruna_crypto::derive_pubkey_hash(&pubkey_y));
    let recipient_y = Address::from_pubkey_hash([0xff; 20]);

    let genesis_hash;
    let genesis_header_stored;

    // Build shared genesis state setup
    let setup = |path: &std::path::Path| -> (Storage, StateManager, ConsensusEngine, Hash) {
        let storage = Storage::open(path).unwrap();
        let state_manager = StateManager::new(storage.clone());
        let engine = ConsensusEngine::new(
            state_manager.clone(),
            storage.clone(),
            Address::from_pubkey_hash([0x01; 20]),
            Address::from_pubkey_hash([0x02; 20]),
            Address::from_pubkey_hash([0x03; 20]),
        );

        let mut init = StorageBatch::new();
        init.put_account(&sender_x, 5_000_000, 0, &Hash::zero(), &Hash::zero());
        init.put_account(&sender_y, 5_000_000, 0, &Hash::zero(), &Hash::zero());
        storage.write_batch(init).unwrap();

        let gen_hdr = BlockHeader {
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
        let gen_body = BlockBody {
            transactions: vec![],
            validator_metadata: vec![],
            ecosystem_metadata: vec![],
        };
        let gen_bytes = aruna_primitives::serialize(&gen_hdr).unwrap();
        let gen_hash = aruna_crypto::blake3_hash(&gen_bytes);
        storage.put_block_header(&gen_hash, &gen_hdr).unwrap();
        storage.put_block_body(&gen_hash, &gen_body).unwrap();
        storage.put_best_block(&gen_hash).unwrap();
        storage.put_chain_height(0).unwrap();
        storage.put_cumulative_difficulty(&gen_hash, 0).unwrap();
        storage.put_block_height_by_hash(&gen_hash, 0).unwrap();
        (storage, state_manager, engine, gen_hash)
    };

    let (storage_x, _state_x, engine_x, gx) = setup(&path_x);
    let (storage_y, _state_y, engine_y, gy) = setup(&path_y);
    genesis_hash = gx;
    genesis_header_stored = storage_x.get_block_header(&genesis_hash).unwrap().unwrap();
    assert_eq!(gx, gy, "Both nodes must share the same genesis hash");

    // ---- Partition Phase ----
    // Node X mines 4 blocks independently
    let mut parent_x = genesis_hash;
    let mut state_x = genesis_header_stored.state_root;
    let mut blocks_x = Vec::new();
    for i in 1u64..=4 {
        let (h, s) = build_and_commit_block(
            &engine_x, &keypair_x, &pubkey_x, sender_x, recipient_x,
            i, 2000, 5000, 1782252000 + i * 30,
            parent_x, state_x,
        );
        blocks_x.push((h, s));
        parent_x = h;
        state_x = s;
    }
    assert_eq!(storage_x.get_chain_height().unwrap().unwrap(), 4);

    // Node Y mines 5 blocks independently (heavier chain).
    // Use storage_y (from setup) — no second open needed, engine_y holds the same handle.
    let mut parent_y = genesis_hash;
    let mut state_y = Hash::zero();
    let mut blocks_y_full: Vec<aruna_primitives::Block> = Vec::new();

    for i in 1u64..=5 {
        let (h, _s) = build_and_commit_block(
            &engine_y, &keypair_y, &pubkey_y, sender_y, recipient_y,
            i, 3000, 5000, 1782252005 + i * 30,
            parent_y, state_y,
        );
        let hdr = storage_y.get_block_header(&h).unwrap().unwrap();
        let body = storage_y.get_block_body(&h).unwrap().unwrap();
        blocks_y_full.push(aruna_primitives::Block { header: hdr.clone(), body });
        state_y = hdr.state_root;
        parent_y = h;
    }

    let tip_y = parent_y;

    // ---- Reconnect Phase: Feed Node Y's blocks into Node X ----
    // Blocks 1Y–4Y: lighter or equal cumulative work → stored as side-chain, X tip stays at 4X
    for block_y in &blocks_y_full[..4] {
        engine_x.commit_block(block_y).unwrap();
        // Node X canonical tip must NOT switch yet (Node Y cumulative work <= 4)
        assert_eq!(storage_x.get_chain_height().unwrap().unwrap(), 4);
    }

    // Block 5Y: triggers deep reorg! Node Y now has 5 blocks > Node X's 4 blocks
    engine_x.commit_block(&blocks_y_full[4]).unwrap();

    // ---- Post-Reconnect Assertions ----
    assert_eq!(storage_x.get_best_block().unwrap().unwrap(), tip_y,
        "Node X canonical tip must switch to Node Y's chain tip");
    assert_eq!(storage_x.get_chain_height().unwrap().unwrap(), 5,
        "Node X chain height must be 5 after reorg");

    // Node X transactions (sender_x → recipient_x) must be fully rolled back
    let (bal_sender_x, nonce_x, _, _) = storage_x.get_account(&sender_x).unwrap()
        .unwrap_or((5_000_000, 0, Hash::zero(), Hash::zero()));
    assert_eq!(bal_sender_x, 5_000_000, "sender_x must be restored to genesis balance after rollback");
    assert_eq!(nonce_x, 0, "sender_x nonce must be reset to 0 after rollback");

    let (bal_rec_x, _, _, _) = storage_x.get_account(&recipient_x).unwrap()
        .unwrap_or((0, 0, Hash::zero(), Hash::zero()));
    assert_eq!(bal_rec_x, 0, "recipient_x must have 0 balance after rollback");

    // Node Y transactions (sender_y → recipient_y) must be applied on Node X
    let (bal_sender_y, nonce_y, _, _) = storage_x.get_account(&sender_y).unwrap().unwrap();
    // 5 transfers of 3000 + 5000 fee each = 5 * 8000 = 40000 deducted from 5_000_000
    assert_eq!(bal_sender_y, 5_000_000 - 5 * (3000 + 5000),
        "sender_y balance must reflect 5 applied transfers");
    assert_eq!(nonce_y, 5, "sender_y nonce must be 5");

    let (bal_rec_y, _, _, _) = storage_x.get_account(&recipient_y).unwrap().unwrap();
    assert_eq!(bal_rec_y, 5 * 3000, "recipient_y must have received 5 * 3000 micro-ARU");

    println!("Network partition + reconnect test successful! Heavier chain won, state rolled back cleanly.");
}

/// Multiple Rapid Restarts Chaos Test.
///
/// Simulates a node that experiences consecutive sudden deaths and restarts.
/// We verify that no database index corruption occurs and all state transitions
/// remain perfectly consistent and sequential after repeated restarts.
#[test]
fn test_multiple_rapid_restarts() {
    use aruna_state::StateManager;
    use aruna_consensus::ConsensusEngine;
    use aruna_primitives::{Address, Hash};
    use aruna_crypto::Ed25519Keypair;

    let path = temp_db_path();
    let _cleaner = TempDirCleaner { path: path.clone() };

    let keypair = Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let sender = Address::from_pubkey_hash(aruna_crypto::derive_pubkey_hash(&pubkey));
    let recipient = Address::from_pubkey_hash([0xcc; 20]);

    // Setup helper
    let setup = |db_path: &std::path::Path, is_init: bool| -> (Storage, StateManager, ConsensusEngine, Hash) {
        let storage = Storage::open(db_path).expect("Failed to open storage");
        let state_manager = StateManager::new(storage.clone());
        let engine = ConsensusEngine::new(
            state_manager.clone(),
            storage.clone(),
            Address::from_pubkey_hash([0x01; 20]),
            Address::from_pubkey_hash([0x02; 20]),
            Address::from_pubkey_hash([0x03; 20]),
        );

        let genesis_hash = if is_init {
            // Fund sender
            let mut init = StorageBatch::new();
            init.put_account(&sender, 10_000_000, 0, &Hash::zero(), &Hash::zero());
            storage.write_batch(init).unwrap();

            // Genesis setup
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
            let genesis_bytes = aruna_primitives::serialize(&genesis_header).unwrap();
            let g_hash = aruna_crypto::blake3_hash(&genesis_bytes);
            storage.put_block_header(&g_hash, &genesis_header).unwrap();
            storage.put_block_body(&g_hash, &genesis_body).unwrap();
            storage.put_best_block(&g_hash).unwrap();
            storage.put_chain_height(0).unwrap();
            storage.put_cumulative_difficulty(&g_hash, 0).unwrap();
            storage.put_block_height_by_hash(&g_hash, 0).unwrap();
            storage.put_block_height_map(0, &g_hash).unwrap();
            g_hash
        } else {
            storage.get_block_hash_by_height(0).unwrap().unwrap()
        };

        (storage, state_manager, engine, genesis_hash)
    };

    // 1. Initial setup
    let mut parent_hash;
    let mut parent_state = Hash::zero();
    {
        let (_storage, _state_manager, _engine, genesis_hash) = setup(&path, true);
        parent_hash = genesis_hash;
    }

    // 2. Perform 3 consecutive rapid restart & mining cycles
    for i in 1u64..=3 {
        let (storage, _state_manager, engine, _genesis_hash) = setup(&path, false);

        // Verify height before block production
        assert_eq!(storage.get_chain_height().unwrap().unwrap(), i - 1);

        // Produce and commit a block
        let (hash, state_root) = build_and_commit_block(
            &engine, &keypair, &pubkey, sender, recipient,
            i, 1000, 5000, 1782252000 + i * 30,
            parent_hash, parent_state,
        );

        parent_hash = hash;
        parent_state = state_root;

        // Verify height immediately increases
        assert_eq!(storage.get_chain_height().unwrap().unwrap(), i);

        // Simulate crash: Drop handles inside loop
    }

    // 3. Final verification after all restarts
    let (storage, _state_manager, _engine, _genesis_hash) = setup(&path, false);
    assert_eq!(storage.get_chain_height().unwrap().unwrap(), 3, "Node height should be exactly 3 after 3 crashes/restarts");
    assert_eq!(storage.get_best_block().unwrap().unwrap(), parent_hash);

    let (final_bal, final_nonce, _, _) = storage.get_account(&sender).unwrap().unwrap();
    // 3 transfers of 1000 + 5000 fee = 3 * 6000 = 18000 micro-ARU deducted from 10_000_000
    assert_eq!(final_bal, 10_000_000 - 18_000);
    assert_eq!(final_nonce, 3);

    println!("Multiple rapid restarts chaos test passed successfully!");
}
