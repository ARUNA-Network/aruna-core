//! Integration test validating the ARUNA state commitment and verification pipeline.
//!
//! Verifies the flow:
//!   Execute Transactions -> State Transitions -> State Root Generation -> Header Embedding -> Verification & Commit
//!
//! Asserts that:
//! 1. Valid transaction execution produces a deterministic state root.
//! 2. A block header containing the correct state root is accepted and committed.
//! 3. A block header containing a modified/corrupted state root is rejected with a validation error,
//!    and the ledger state changes are completely rolled back (no state corruption).

use aruna_primitives::{Block, BlockBody, BlockHeader, Hash, Difficulty, Address, Nonce, TransactionPayload, TransactionEnvelope, SignatureType};
use aruna_storage::{Storage, StorageBatch};
use aruna_state::StateManager;
use aruna_consensus::{ConsensusEngine, ConsensusError};
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
    path.push(format!("aruna_state_commitment_{}_{}", suffix, nanos));
    path
}

fn initialize_ledger_state(path: &std::path::Path, sender: &Address) -> (Storage, StateManager, ConsensusEngine, Hash) {
    let storage = Storage::open(path).expect("Failed to open storage");
    let state_manager = StateManager::new(storage.clone());
    let consensus_engine = ConsensusEngine::new(
        state_manager.clone(),
        storage.clone(),
        Address::from_pubkey_hash([0x01; 20]),
        Address::from_pubkey_hash([0x02; 20]),
        Address::from_pubkey_hash([0x03; 20]),
    );

    // Fund the sender in genesis
    let mut init_batch = StorageBatch::new();
    init_batch.put_account(sender, 10_000_000, 0, &Hash::zero(), &Hash::zero());
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

    (storage, state_manager, consensus_engine, genesis_hash)
}

#[test]
fn test_state_commitment_pipeline() {
    let path = temp_db_path("state_commit");
    let _cleaner = TempDirCleaner { path: path.clone() };

    let keypair = aruna_crypto::Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let pkh = aruna_crypto::derive_pubkey_hash(&pubkey);
    let sender = Address::from_pubkey_hash(pkh);

    let (_storage, state_manager, engine, genesis_hash) = initialize_ledger_state(&path, &sender);

    // Verify initial state
    let initial_acc = state_manager.get_account(&sender).unwrap().unwrap();
    assert_eq!(initial_acc.balance, 10_000_000);
    assert_eq!(initial_acc.nonce.0, 0);

    // 1. Create and Sign Transactions
    println!("Step 1: Creating and signing transactions...");
    let recipient_1 = Address::from_pubkey_hash([0x11; 20]);
    let recipient_2 = Address::from_pubkey_hash([0x22; 20]);

    let payload_1 = TransactionPayload {
        nonce: Nonce(1),
        sender,
        recipient: recipient_1,
        amount: 200_000,
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

    let payload_2 = TransactionPayload {
        nonce: Nonce(2),
        sender,
        recipient: recipient_2,
        amount: 300_000,
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

    let body = BlockBody {
        transactions: vec![tx_1, tx_2],
        validator_metadata: vec![],
        ecosystem_metadata: vec![],
    };

    // 2. Execute & Calculate State Root
    println!("Step 2: Executing transactions and computing state root...");
    let merkle_root = ConsensusEngine::calculate_merkle_root(&body.transactions).unwrap();
    let header_template = BlockHeader {
        version: 1,
        prev_block_hash: genesis_hash,
        merkle_root,
        state_root: Hash::zero(), // placeholder
        timestamp: 1782252100,
        difficulty: Difficulty(504381424),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let block_template = Block {
        header: header_template,
        body: body.clone(),
    };

    // Dry-run execution to calculate state root commitment
    let calculated_state_root = engine.calculate_state_root(Hash::zero(), &block_template).unwrap();
    println!("Calculated State Root: {:?}", calculated_state_root);
    assert_ne!(calculated_state_root, Hash::zero());

    // 3. Byzantine Verification: Embed corrupted state root into BlockHeader
    println!("Step 3: Verifying that wrong state root commitment is rejected...");
    let corrupted_header = BlockHeader {
        state_root: Hash::new([0xFF; 32]), // mutated state root
        ..header_template
    };
    let corrupted_block = Block {
        header: corrupted_header,
        body: body.clone(),
    };

    // Attempt to commit block with incorrect state root
    let commit_result = engine.commit_block(&corrupted_block);
    assert!(commit_result.is_err());
    match commit_result {
        Err(ConsensusError::Validation(msg)) => {
            println!("Rejection verified: {}", msg);
            assert!(msg.contains("State root mismatch"));
        }
        other => panic!("Expected state root validation error, got {:?}", other),
    }

    // Verify that state changes were rolled back and sender balance remains intact
    let post_fail_acc = state_manager.get_account(&sender).unwrap().unwrap();
    assert_eq!(post_fail_acc.balance, 10_000_000, "State must rollback on validation failure");
    assert_eq!(post_fail_acc.nonce.0, 0);

    // 4. Conformance Verification: Embed correct state root into BlockHeader
    println!("Step 4: Verifying that correct state root commitment is accepted...");
    let valid_header = BlockHeader {
        state_root: calculated_state_root,
        ..header_template
    };
    let valid_block = Block {
        header: valid_header,
        body,
    };

    // Commit block with correct state root
    let commit_success = engine.commit_block(&valid_block);
    assert!(commit_success.is_ok());
    let block_hash = commit_success.unwrap();
    println!("Block successfully committed with hash: {:?}", block_hash);

    // Assert that the ledger state is updated and sender balance is correctly mutated
    let post_success_acc = state_manager.get_account(&sender).unwrap().unwrap();
    assert_eq!(post_success_acc.balance, 9_490_000); // 10M - 200K - 300K - 10K fees
    assert_eq!(post_success_acc.nonce.0, 2);

    println!("State commitment and verification pipeline successfully verified!");
}
