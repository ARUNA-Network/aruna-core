//! Consensus validation engine for the ARUNA Network.
//! Conforms to specifications defined in docs/protocol/consensus.md, block.md, and transaction.md.

use aruna_primitives::{Block, BlockHeader, TransactionEnvelope, Hash, serialize, SignatureType};
use aruna_state::{StateManager, StateError};
use aruna_storage::{Storage, StorageBatch, StorageError};
use aruna_crypto::{CryptoError, Ed25519Verifier, derive_pubkey_hash};
use thiserror::Error;

/// Error type for block and transaction consensus validation.
#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("Database error: {0}")]
    Database(#[from] StorageError),
    #[error("State validation error: {0}")]
    State(#[from] StateError),
    #[error("Cryptography error: {0}")]
    Cryptography(#[from] CryptoError),
    #[error("Signature length must be {expected} bytes, got {got}")]
    InvalidSignatureLength { expected: usize, got: usize },
    #[error("Address mismatch: transaction sender does not match public key hash")]
    AddressMismatch,
    #[error("Block size exceeded: maximum is 2 MB, block is {size} bytes")]
    BlockSizeExceeded { size: usize },
    #[error("Invalid Merkle Root: expected {expected:?}, got {got:?}")]
    InvalidMerkleRoot { expected: Hash, got: Hash },
    #[error("Invalid Parent Hash: expected {expected:?}, got {got:?}")]
    InvalidParentHash { expected: Hash, got: Hash },
    #[error("Invalid compact difficulty target")]
    InvalidDifficulty,
    #[error("Invalid Timestamp: block timestamp {timestamp} must be greater than minimum {min_timestamp}")]
    InvalidTimestamp { timestamp: u64, min_timestamp: u64 },
    #[error("Consensus violation: {0}")]
    Validation(String),
}

/// The consensus validation coordinator.
pub struct ConsensusEngine {
    state_manager: StateManager,
    storage: Storage,
}

impl ConsensusEngine {
    /// Create a new ConsensusEngine instance.
    pub fn new(state_manager: StateManager, storage: Storage) -> Self {
        Self {
            state_manager,
            storage,
        }
    }

    /// Calculate the block reward splits (Miner 70%, Validator 25%, Treasury 5%)
    /// incorporating the 4-year halving interval (4,204,800 blocks).
    pub fn calculate_reward(height: u64) -> (u64, u64, u64) {
        let era = height / 4_204_800;
        if era >= 64 {
            return (0, 0, 0); // Max supply bounds reached
        }
        // Base Genesis Reward: 25 ARU = 25,000,000 micro-ARU. Halves every era.
        let total_reward = 25_000_000_u64 >> era;

        let miner_reward = (total_reward * 70) / 100;
        let validator_reward = (total_reward * 25) / 100;
        // Sweeps any dust remainder into the treasury contract allocation
        let treasury_reward = total_reward - miner_reward - validator_reward;

        (miner_reward, validator_reward, treasury_reward)
    }

    /// Calculate the deterministic BLAKE3 Merkle root of block transactions.
    pub fn calculate_merkle_root(txs: &[TransactionEnvelope]) -> Result<Hash, ConsensusError> {
        if txs.is_empty() {
            return Ok(Hash::zero());
        }

        let mut leaves: Vec<Hash> = txs
            .iter()
            .map(|tx| {
                let bytes = serialize(tx).unwrap();
                aruna_crypto::blake3_hash(&bytes)
            })
            .collect();

        while leaves.len() > 1 {
            let mut next_level = Vec::with_capacity((leaves.len() + 1) / 2);
            for chunk in leaves.chunks(2) {
                let mut concat = [0u8; 64];
                concat[0..32].copy_from_slice(&chunk[0].0);
                if chunk.len() == 2 {
                    concat[32..64].copy_from_slice(&chunk[1].0);
                } else {
                    concat[32..64].copy_from_slice(&chunk[0].0); // Duplicate odd leaf
                }
                next_level.push(aruna_crypto::blake3_hash(&concat));
            }
            leaves = next_level;
        }

        Ok(leaves[0])
    }

    /// Validates a single transaction, including cryptographic signatures and ledger account states.
    pub fn validate_transaction(&self, tx: &TransactionEnvelope) -> Result<(), ConsensusError> {
        // 1. Verify cryptographic signature
        match tx.signature_type {
            SignatureType::Ed25519 => {
                if tx.public_key.len() != 32 {
                    return Err(ConsensusError::Validation("Invalid public key length".to_string()));
                }
                if tx.signature.len() != 64 {
                    return Err(ConsensusError::InvalidSignatureLength {
                        expected: 64,
                        got: tx.signature.len(),
                    });
                }

                // Check that the public key matches the transaction sender address
                let pkh = derive_pubkey_hash(&tx.public_key);
                let derived_sender = aruna_primitives::Address::from_pubkey_hash(pkh);
                if derived_sender != tx.payload.sender {
                    return Err(ConsensusError::AddressMismatch);
                }

                // Verify Ed25519 signature
                let mut pubkey_array = [0u8; 32];
                pubkey_array.copy_from_slice(&tx.public_key);

                let mut sig_array = [0u8; 64];
                sig_array.copy_from_slice(&tx.signature);

                let msg_bytes = serialize(&tx.payload).map_err(|e| ConsensusError::Cryptography(CryptoError::Serialization(e.to_string())))?;
                Ed25519Verifier::verify(&pubkey_array, &msg_bytes, &sig_array)?;
            }
            SignatureType::Secp256k1 => {
                // EVM secp256k1 signatures are planned in Phase 6 / EVM integration
                return Err(ConsensusError::Validation("Secp256k1 signature validation is not yet implemented".to_string()));
            }
        }

        // 2. Dry-run transition logic against the ledger state
        let mut dry_run_batch = StorageBatch::new();
        self.state_manager.apply_transaction(tx, &mut dry_run_batch)?;

        Ok(())
    }

    /// Validates an entire block header, Merkle tree alignments, and transaction list constraints.
    pub fn validate_block(&self, block: &Block) -> Result<(), ConsensusError> {
        // 1. Enforce block size limit (< 2 MB)
        let block_bytes = serialize(block).map_err(|e| ConsensusError::Database(StorageError::Format(e.to_string())))?;
        if block_bytes.len() > 2 * 1024 * 1024 {
            return Err(ConsensusError::BlockSizeExceeded {
                size: block_bytes.len(),
            });
        }

        // 2. Validate previous block hash matches database best block
        if block.header.version > 1 {
            let best_hash = self
                .storage
                .get_best_block()?
                .ok_or_else(|| ConsensusError::Validation("No parent block header found".to_string()))?;
            if block.header.prev_block_hash != best_hash {
                return Err(ConsensusError::InvalidParentHash {
                    expected: best_hash,
                    got: block.header.prev_block_hash,
                });
            }
        }

        // 3. Verify Merkle root of transaction bodies
        let derived_merkle = Self::calculate_merkle_root(&block.body.transactions)?;
        if block.header.merkle_root != derived_merkle {
            return Err(ConsensusError::InvalidMerkleRoot {
                expected: derived_merkle,
                got: block.header.merkle_root,
            });
        }

        // 4. Validate all transaction envelopes inside the block
        for tx in &block.body.transactions {
            self.validate_transaction(tx)?;
        }

        Ok(())
    }

    /// Verifies block compact difficulty target transitions (nBits).
    pub fn verify_difficulty(&self, parent: &BlockHeader, current: &BlockHeader) -> Result<(), ConsensusError> {
        if current.difficulty != parent.difficulty {
            // Difficulty transition verification WMA equation is planned in Kalimantan/PoW Sprint
            return Err(ConsensusError::InvalidDifficulty);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aruna_primitives::{Address, Nonce, TransactionPayload};
    use aruna_crypto::Ed25519Keypair;

    fn temp_db_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("aruna_consensus_test_{}", time));
        path
    }

    #[test]
    fn test_calculate_reward_halvings() {
        // Era 1 (block height 0)
        let (m1, v1, t1) = ConsensusEngine::calculate_reward(0);
        assert_eq!(m1, 17_500_000); // 70%
        assert_eq!(v1, 6_250_000);  // 25%
        assert_eq!(t1, 1_250_000);  // 5% (sweeps dust)

        // Era 2 (after 4,204,800 blocks)
        let (m2, v2, t2) = ConsensusEngine::calculate_reward(4_204_800);
        assert_eq!(m2, 8_750_000);
        assert_eq!(v2, 3_125_000);
        assert_eq!(t2, 625_000);
    }

    #[test]
    fn test_merkle_root_calculation() {
        let tx = TransactionEnvelope {
            payload: TransactionPayload {
                nonce: Nonce(1),
                sender: Address::from_pubkey_hash([0x11; 20]),
                recipient: Address::from_pubkey_hash([0x22; 20]),
                amount: 10,
                fee: 1,
                gas_limit: 0,
                gas_price: 0,
                data: vec![],
            },
            signature_type: SignatureType::Ed25519,
            signature: vec![0; 64],
            public_key: vec![0; 32],
        };

        // Single transaction
        let root = ConsensusEngine::calculate_merkle_root(&[tx.clone()]).unwrap();
        let serialized_tx = serialize(&tx).unwrap();
        let expected = aruna_crypto::blake3_hash(&serialized_tx);
        assert_eq!(root, expected);
    }

    #[test]
    fn test_validate_transaction_signature_verification() {
        let path = temp_db_path();
        {
            let storage = Storage::open(&path).unwrap();
            let state_manager = StateManager::new(storage.clone());
            let engine = ConsensusEngine::new(state_manager.clone(), storage);

            // Generate real signing credentials
            let keypair = Ed25519Keypair::generate();
            let pubkey = keypair.public_key_bytes();
            
            let pkh = derive_pubkey_hash(&pubkey);
            let sender_addr = Address::from_pubkey_hash(pkh);
            let recipient_addr = Address::from_pubkey_hash([0x22; 20]);

            // Fund sender
            state_manager.put_account(&sender_addr, &aruna_state::Account::new(10000, Nonce(0))).unwrap();

            let payload = TransactionPayload {
                nonce: Nonce(1),
                sender: sender_addr,
                recipient: recipient_addr,
                amount: 1000,
                fee: 100,
                gas_limit: 0,
                gas_price: 0,
                data: vec![],
            };

            let sig = keypair.sign(&serialize(&payload).unwrap());

            let tx = TransactionEnvelope {
                payload,
                signature_type: SignatureType::Ed25519,
                signature: sig.to_vec(),
                public_key: pubkey.to_vec(),
            };

            // Validation succeeds
            let result = engine.validate_transaction(&tx);
            assert!(result.is_ok());

            // Validation fails if public key is altered (sender address mismatch)
            let mut corrupted_tx = tx.clone();
            corrupted_tx.public_key = vec![0; 32];
            let result_corrupted = engine.validate_transaction(&corrupted_tx);
            assert!(matches!(result_corrupted, Err(ConsensusError::AddressMismatch)));
        }
        let _ = std::fs::remove_dir_all(&path);
    }
}
