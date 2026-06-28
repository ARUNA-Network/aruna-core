//! Integration test suite verifying the correctness of all ARUNA consensus validation rules.
//!
//! Asserts that the consensus engine rejects:
//! 1. Blocks exceeding the 2 MB size limit (`BlockSizeExceeded`).
//! 2. Blocks containing invalid transaction Merkle roots (`InvalidMerkleRoot`).
//! 3. Blocks containing transactions with wrong/mutated cryptographic signatures.
//! 4. Blocks containing transactions with out-of-order nonces (`NonceMismatch`).
//! 5. Blocks containing transactions with fees below the minimum floor (`2280 micro-ARU`).
//! 6. Blocks with timestamps older than or equal to the parent block (`InvalidTimestamp`).
//! 7. Blocks with timestamps in the future exceeding the 2-hour drift limit.

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
    path.push(format!("aruna_consensus_correctness_{}_{}", suffix, nanos));
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
        timestamp: 1782252000, // Fixed baseline time
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
fn test_consensus_correctness_rules() {
    let path = temp_db_path("consensus_rules");
    let _cleaner = TempDirCleaner { path: path.clone() };

    let keypair = aruna_crypto::Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let pkh = aruna_crypto::derive_pubkey_hash(&pubkey);
    let sender = Address::from_pubkey_hash(pkh);
    let recipient = Address::from_pubkey_hash([0x09; 20]);

    let (_storage, _state_manager, engine, genesis_hash) = initialize_ledger_state(&path, &sender);

    // Build a template transaction with valid defaults
    let make_envelope = |nonce: u64, fee: u64, mutate_sig: bool| -> TransactionEnvelope {
        let payload = TransactionPayload {
            nonce: Nonce(nonce),
            sender,
            recipient,
            amount: 50_000,
            fee,
            gas_limit: 0,
            gas_price: 0,
            data: vec![],
        };
        let payload_bytes = serialize(&payload).unwrap();
        let mut sig = keypair.sign(&payload_bytes).to_vec();
        if mutate_sig && !sig.is_empty() {
            sig[0] ^= 0xFF; // corrupt signature
        }
        TransactionEnvelope {
            payload,
            signature_type: SignatureType::Ed25519,
            signature: sig,
            public_key: pubkey.to_vec(),
        }
    };

    // ── 1. Block Size Exceeded Rejection ────────────────────────────────────
    println!("Testing block size exceeded rejection...");
    let huge_tx = TransactionEnvelope {
        public_key: vec![0u8; 2 * 1024 * 1024 + 100], // size > 2 MB
        ..make_envelope(1, 5000, false)
    };
    let huge_body = BlockBody {
        transactions: vec![huge_tx],
        validator_metadata: vec![],
        ecosystem_metadata: vec![],
    };
    let huge_merkle = ConsensusEngine::calculate_merkle_root(&huge_body.transactions).unwrap();
    let block_huge = Block {
        header: BlockHeader {
            version: 1,
            prev_block_hash: genesis_hash,
            merkle_root: huge_merkle,
            state_root: Hash::zero(),
            timestamp: 1782252100,
            difficulty: Difficulty(504381424),
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        },
        body: huge_body,
    };
    let res_size = engine.validate_block_header_only(&block_huge);
    assert!(matches!(res_size, Err(ConsensusError::BlockSizeExceeded { .. })));

    // ── 2. Invalid Merkle Root Rejection ───────────────────────────────────
    println!("Testing invalid Merkle root rejection...");
    let tx_valid = make_envelope(1, 5000, false);
    let valid_body = BlockBody {
        transactions: vec![tx_valid.clone()],
        validator_metadata: vec![],
        ecosystem_metadata: vec![],
    };
    let block_invalid_merkle = Block {
        header: BlockHeader {
            version: 1,
            prev_block_hash: genesis_hash,
            merkle_root: Hash::new([0xAA; 32]), // modified Merkle root
            state_root: Hash::zero(),
            timestamp: 1782252100,
            difficulty: Difficulty(504381424),
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        },
        body: valid_body.clone(),
    };
    let res_merkle = engine.validate_block_header_only(&block_invalid_merkle);
    assert!(matches!(res_merkle, Err(ConsensusError::InvalidMerkleRoot { .. })));

    // ── 3. Wrong Signature Rejection ────────────────────────────────────────
    println!("Testing wrong signature rejection...");
    let tx_bad_sig = make_envelope(1, 5000, true); // mutated signature
    let bad_sig_body = BlockBody {
        transactions: vec![tx_bad_sig],
        validator_metadata: vec![],
        ecosystem_metadata: vec![],
    };
    let bad_sig_merkle = ConsensusEngine::calculate_merkle_root(&bad_sig_body.transactions).unwrap();
    let block_bad_sig = Block {
        header: BlockHeader {
            version: 1,
            prev_block_hash: genesis_hash,
            merkle_root: bad_sig_merkle,
            state_root: Hash::zero(),
            timestamp: 1782252100,
            difficulty: Difficulty(504381424),
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        },
        body: bad_sig_body,
    };
    let res_sig = engine.validate_block_header_only(&block_bad_sig);
    assert!(res_sig.is_err(), "Block validation must reject bad transaction signature");

    // ── 4. Wrong Nonce Rejection ───────────────────────────────────────────
    println!("Testing wrong nonce rejection...");
    let tx_bad_nonce = make_envelope(5, 5000, false); // expected Nonce(1), got Nonce(5)
    let bad_nonce_body = BlockBody {
        transactions: vec![tx_bad_nonce],
        validator_metadata: vec![],
        ecosystem_metadata: vec![],
    };
    let bad_nonce_merkle = ConsensusEngine::calculate_merkle_root(&bad_nonce_body.transactions).unwrap();
    let block_bad_nonce = Block {
        header: BlockHeader {
            version: 1,
            prev_block_hash: genesis_hash,
            merkle_root: bad_nonce_merkle,
            state_root: Hash::zero(),
            timestamp: 1782252100,
            difficulty: Difficulty(504381424),
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        },
        body: bad_nonce_body,
    };
    let res_nonce = engine.commit_block(&block_bad_nonce);
    assert!(res_nonce.is_err(), "Block commit must reject invalid nonce execution");
    match res_nonce {
        Err(ConsensusError::Validation(msg)) => {
            assert!(msg.contains("Nonce mismatch"));
        }
        other => panic!("Expected Validation Nonce Mismatch error, got {:?}", other),
    }

    // ── 5. Wrong Fee Rejection ──────────────────────────────────────────────
    println!("Testing wrong fee rejection...");
    let tx_low_fee = make_envelope(1, 100, false); // fee of 100 is below the 2280 micro-ARU floor
    let low_fee_body = BlockBody {
        transactions: vec![tx_low_fee],
        validator_metadata: vec![],
        ecosystem_metadata: vec![],
    };
    let low_fee_merkle = ConsensusEngine::calculate_merkle_root(&low_fee_body.transactions).unwrap();
    let block_low_fee = Block {
        header: BlockHeader {
            version: 1,
            prev_block_hash: genesis_hash,
            merkle_root: low_fee_merkle,
            state_root: Hash::zero(),
            timestamp: 1782252100,
            difficulty: Difficulty(504381424),
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        },
        body: low_fee_body,
    };
    let res_fee = engine.validate_block_header_only(&block_low_fee);
    println!("Fee rejection result: {:?}", res_fee);
    assert!(res_fee.is_err(), "Block validation must reject low transaction fee");
    match res_fee {
        Err(ConsensusError::Validation(msg)) => {
            assert!(msg.contains("fee too low") || msg.contains("Transaction fee too low"));
        }
        other => panic!("Expected low fee validation error, got {:?}", other),
    }

    // ── 6. Wrong Timestamp Rejection (Older/Equal than parent) ──────────────
    println!("Testing wrong timestamp rejection (older than parent block)...");
    let normal_merkle = ConsensusEngine::calculate_merkle_root(&valid_body.transactions).unwrap();
    let block_old_timestamp = Block {
        header: BlockHeader {
            version: 1,
            prev_block_hash: genesis_hash,
            merkle_root: normal_merkle,
            state_root: Hash::zero(),
            timestamp: 1782251999, // Genesis is 1782252000; older!
            difficulty: Difficulty(504381424),
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        },
        body: valid_body.clone(),
    };
    let res_time1 = engine.validate_block_header_only(&block_old_timestamp);
    println!("Old timestamp rejection result: {:?}", res_time1);
    assert!(matches!(res_time1, Err(ConsensusError::InvalidTimestamp { .. })));

    // ── 7. Future Drift Timestamp Rejection ──────────────────────────────────
    println!("Testing future drift timestamp rejection...");
    let block_future_timestamp = Block {
        header: BlockHeader {
            version: 1,
            prev_block_hash: genesis_hash,
            merkle_root: normal_merkle,
            state_root: Hash::zero(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() + 10_000, // > 2 hours in the future!
            difficulty: Difficulty(504381424),
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        },
        body: valid_body,
    };
    let res_time2 = engine.validate_block_header_only(&block_future_timestamp);
    println!("Future timestamp rejection result: {:?}", res_time2);
    assert!(res_time2.is_err(), "Block validation must reject timestamp with future drift");
    match res_time2 {
        Err(ConsensusError::Validation(msg)) => {
            assert!(msg.contains("too far in the future"));
        }
        other => panic!("Expected future timestamp validation error, got {:?}", other),
    }

    println!("All consensus correctness validation rules successfully verified!");
}
