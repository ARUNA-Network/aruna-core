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
        let engine = ConsensusEngine::new(state_manager.clone(), storage.clone());

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
