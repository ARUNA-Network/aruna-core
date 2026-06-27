//! Consensus validation engine for the ARUNA Network.
//! Conforms to specifications defined in docs/protocol/consensus.md, block.md, and transaction.md.

use aruna_primitives::{Block, BlockHeader, BlockBody, TransactionEnvelope, Hash, serialize, SignatureType, Address};
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
#[derive(Clone)]
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

    /// Produces a new block extending the current best block tip with a list of transactions.
    pub fn produce_block(&self, transactions: Vec<TransactionEnvelope>) -> Result<Block, ConsensusError> {
        let best_hash = self.storage.get_best_block()?
            .ok_or_else(|| ConsensusError::Validation("No best block found".to_string()))?;
        let best_header = self.storage.get_block_header(&best_hash)?
            .ok_or_else(|| ConsensusError::Validation("Best block header not found".to_string()))?;
        let mut timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Enforce strictly increasing block timestamps
        if timestamp <= best_header.timestamp {
            timestamp = best_header.timestamp + 1;
        }

        let body = BlockBody {
            transactions,
            validator_metadata: vec![],
            ecosystem_metadata: vec![],
        };

        let merkle_root = Self::calculate_merkle_root(&body.transactions)?;

        let header = BlockHeader {
            version: 1,
            prev_block_hash: best_hash,
            merkle_root,
            state_root: Hash::zero(),
            timestamp,
            difficulty: best_header.difficulty,
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        };

        let mut block = Block { header, body };

        // Calculate dry-run state root and assign to header
        let parent_state_root = best_header.state_root;
        let final_state_root = self.calculate_state_root(parent_state_root, &block)?;
        block.header.state_root = final_state_root;

        // Perform consensus dry-run validations on the new block
        self.validate_block(&block)?;

        Ok(block)
    }

    /// Commits a block to the RocksDB database, executing its transactions, updating ledger state, and distributing rewards.
    pub fn commit_block(&self, block: &Block) -> Result<Hash, ConsensusError> {
        // Dry-run validate the block state transitions and consensus criteria
        self.validate_block(block)?;

        // 1. Serialize block header and calculate block hash
        let header_bytes = serialize(&block.header)
            .map_err(|e| ConsensusError::Database(StorageError::Format(e.to_string())))?;
        let block_hash = aruna_crypto::blake3_hash(&header_bytes);

        // 2. Open a storage batch
        let mut batch = StorageBatch::new();

        // 3. Keep a cache of modified accounts to prevent overwriting updates
        let mut account_cache: std::collections::HashMap<Address, aruna_state::Account> = std::collections::HashMap::new();

        // Local helper closure to get account from cache or database (without capturing to satisfy borrow checker)
        let get_account_local = |addr: &Address, cache: &std::collections::HashMap<Address, aruna_state::Account>, state_mgr: &StateManager| -> Result<aruna_state::Account, ConsensusError> {
            if let Some(acc) = cache.get(addr) {
                Ok(acc.clone())
            } else {
                let acc = state_mgr.get_account(addr)?
                    .unwrap_or_else(|| aruna_state::Account::new(0, aruna_primitives::Nonce::zero()));
                Ok(acc)
            }
        };

        // 4. Execute all transactions in the block body
        for (tx_idx, tx) in block.body.transactions.iter().enumerate() {
            let sender_addr = tx.payload.sender;
            let recipient_addr = tx.payload.recipient;

            let mut sender = get_account_local(&sender_addr, &account_cache, &self.state_manager)?;
            let mut recipient = get_account_local(&recipient_addr, &account_cache, &self.state_manager)?;

            // Validate Nonce
            let expected_nonce = sender.nonce.increment();
            if tx.payload.nonce != expected_nonce {
                return Err(ConsensusError::Validation(format!(
                    "Nonce mismatch for sender {:?}: expected {:?}, got {:?}",
                    sender_addr, expected_nonce, tx.payload.nonce
                )));
            }

            // Validate Balance
            let total_required = tx.payload.amount.checked_add(tx.payload.fee)
                .ok_or_else(|| ConsensusError::Validation("Transaction amount/fee overflow".to_string()))?;
            
            if sender.balance < total_required {
                return Err(ConsensusError::Validation(format!(
                    "Insufficient balance for sender {:?}: has {}, requires {}",
                    sender_addr, sender.balance, total_required
                )));
            }

            // Apply state changes to cache
            sender.balance -= total_required;
            sender.nonce = tx.payload.nonce;

            recipient.balance = recipient.balance.checked_add(tx.payload.amount)
                .ok_or_else(|| ConsensusError::Validation("Recipient balance overflow".to_string()))?;

            account_cache.insert(sender_addr, sender);
            account_cache.insert(recipient_addr, recipient);

            // Index transaction location: tx_hash -> (block_hash, tx_index)
            let tx_bytes = serialize(tx)
                .map_err(|e| ConsensusError::Database(StorageError::Format(e.to_string())))?;
            let tx_hash = aruna_crypto::blake3_hash(&tx_bytes);
            batch.put_tx_index(&tx_hash, &block_hash, tx_idx as u32);
        }

        // 5. Distribute block rewards and accumulated fees
        let current_height = self.storage.get_chain_height()?.unwrap_or(0);
        let new_height = current_height + 1;

        let total_fees: u64 = block.body.transactions.iter().map(|tx| tx.payload.fee).sum();
        let (miner_reward, validator_reward, treasury_reward) = Self::calculate_reward(new_height);

        // Miner 70%, Validator 25%, Treasury 5%
        let total_miner_payout = miner_reward + (total_fees * 70 / 100);
        let total_validator_payout = validator_reward + (total_fees * 25 / 100);
        
        let total_pool = miner_reward + validator_reward + treasury_reward + total_fees;
        let total_treasury_payout = total_pool - total_miner_payout - total_validator_payout;

        // Resolve system addresses
        let (_, miner_addr) = Address::from_bech32m("sum1qyqszqgpqyqszqgpqyqszqgpqyqszqgpe6sslr").unwrap();
        let (_, validator_addr) = Address::from_bech32m("sum1qgpqyqszqgpqyqszqgpqyqszqgpqyqszg7k454").unwrap();
        let (_, treasury_addr) = Address::from_bech32m("sumc1qszqgpqyqszqgpqyqszqgpqyqszqgpqypa49fy").unwrap();

        // Credit Miner
        let mut miner = get_account_local(&miner_addr, &account_cache, &self.state_manager)?;
        miner.balance = miner.balance.checked_add(total_miner_payout)
            .ok_or_else(|| ConsensusError::Validation("Miner balance overflow".to_string()))?;
        account_cache.insert(miner_addr, miner);

        // Credit Validator
        let mut validator = get_account_local(&validator_addr, &account_cache, &self.state_manager)?;
        validator.balance = validator.balance.checked_add(total_validator_payout)
            .ok_or_else(|| ConsensusError::Validation("Validator balance overflow".to_string()))?;
        account_cache.insert(validator_addr, validator);

        // Credit Treasury
        let mut treasury = get_account_local(&treasury_addr, &account_cache, &self.state_manager)?;
        treasury.balance = treasury.balance.checked_add(total_treasury_payout)
            .ok_or_else(|| ConsensusError::Validation("Treasury balance overflow".to_string()))?;
        account_cache.insert(treasury_addr, treasury);

        // 6. Write all modified accounts back to the storage batch
        for (addr, acc) in account_cache {
            batch.put_account(&addr, acc.balance, acc.nonce.0, &acc.code_hash, &acc.storage_root);
        }

        // 7. Write block header, body, height mapping, and hash-to-height mapping to storage batch
        batch.put_block_height_map(new_height, &block_hash);
        batch.put_block_height_by_hash(&block_hash, new_height);

        // 8. Commit storage batch atomically
        self.storage.write_batch(batch)?;

        // 9. Update tip metadata indexes
        self.storage.put_block_header(&block_hash, &block.header)?;
        self.storage.put_block_body(&block_hash, &block.body)?;
        self.storage.put_best_block(&block_hash)?;
        self.storage.put_chain_height(new_height)?;

        Ok(block_hash)
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

    /// Computes the state root hash deterministically given parent state root and modified accounts list.
    pub fn compute_state_root_from_updates(parent_state_root: Hash, modified_accounts: &std::collections::HashMap<Address, aruna_state::Account>) -> Result<Hash, ConsensusError> {
        if modified_accounts.is_empty() {
            return Ok(parent_state_root);
        }

        // Sort addresses lexicographically for determinism
        let mut sorted_keys: Vec<&Address> = modified_accounts.keys().collect();
        sorted_keys.sort_unstable();

        let mut updates_bytes = Vec::new();
        for addr in sorted_keys {
            let acc = modified_accounts.get(addr).unwrap();
            let addr_bytes = serialize(addr).map_err(|e| ConsensusError::Database(StorageError::Format(e.to_string())))?;
            let acc_bytes = serialize(acc).map_err(|e| ConsensusError::Database(StorageError::Format(e.to_string())))?;
            updates_bytes.extend_from_slice(&addr_bytes);
            updates_bytes.extend_from_slice(&acc_bytes);
        }

        let block_updates_hash = aruna_crypto::blake3_hash(&updates_bytes);

        let mut concat = [0u8; 64];
        concat[0..32].copy_from_slice(&parent_state_root.0);
        concat[32..64].copy_from_slice(&block_updates_hash.0);

        Ok(aruna_crypto::blake3_hash(&concat))
    }

    /// Dry-runs the block state transitions and calculates the resulting state root hash.
    pub fn calculate_state_root(&self, parent_state_root: Hash, block: &Block) -> Result<Hash, ConsensusError> {
        let mut account_cache: std::collections::HashMap<Address, aruna_state::Account> = std::collections::HashMap::new();

        let get_account_local = |addr: &Address, cache: &std::collections::HashMap<Address, aruna_state::Account>, state_mgr: &StateManager| -> Result<aruna_state::Account, ConsensusError> {
            if let Some(acc) = cache.get(addr) {
                Ok(acc.clone())
            } else {
                let acc = state_mgr.get_account(addr)?
                    .unwrap_or_else(|| aruna_state::Account::new(0, aruna_primitives::Nonce::zero()));
                Ok(acc)
            }
        };

        // 1. Dry-run execute all transactions
        for tx in &block.body.transactions {
            let sender_addr = tx.payload.sender;
            let recipient_addr = tx.payload.recipient;

            let mut sender = get_account_local(&sender_addr, &account_cache, &self.state_manager)?;
            let mut recipient = get_account_local(&recipient_addr, &account_cache, &self.state_manager)?;

            let expected_nonce = sender.nonce.increment();
            if tx.payload.nonce != expected_nonce {
                return Err(ConsensusError::Validation(format!(
                    "Nonce mismatch for sender {:?}: expected {:?}, got {:?}",
                    sender_addr, expected_nonce, tx.payload.nonce
                )));
            }

            let total_required = tx.payload.amount.checked_add(tx.payload.fee)
                .ok_or_else(|| ConsensusError::Validation("Transaction amount/fee overflow".to_string()))?;
            
            if sender.balance < total_required {
                return Err(ConsensusError::Validation(format!(
                    "Insufficient balance for sender {:?}: has {}, requires {}",
                    sender_addr, sender.balance, total_required
                )));
            }

            sender.balance -= total_required;
            sender.nonce = tx.payload.nonce;

            recipient.balance = recipient.balance.checked_add(tx.payload.amount)
                .ok_or_else(|| ConsensusError::Validation("Recipient balance overflow".to_string()))?;

            account_cache.insert(sender_addr, sender);
            account_cache.insert(recipient_addr, recipient);
        }

        // 2. Dry-run block rewards
        let next_height = if block.header.prev_block_hash == Hash::zero() {
            0
        } else {
            let parent_height = self.storage.get_block_height_by_hash(&block.header.prev_block_hash)?
                .ok_or_else(|| ConsensusError::Validation(format!("Parent block height not found for hash {:?}", block.header.prev_block_hash)))?;
            parent_height + 1
        };

        let total_fees: u64 = block.body.transactions.iter().map(|tx| tx.payload.fee).sum();
        let (miner_reward, validator_reward, treasury_reward) = Self::calculate_reward(next_height);

        let total_miner_payout = miner_reward + (total_fees * 70 / 100);
        let total_validator_payout = validator_reward + (total_fees * 25 / 100);
        
        let total_pool = miner_reward + validator_reward + treasury_reward + total_fees;
        let total_treasury_payout = total_pool - total_miner_payout - total_validator_payout;

        let (_, miner_addr) = Address::from_bech32m("sum1qyqszqgpqyqszqgpqyqszqgpqyqszqgpe6sslr").unwrap();
        let (_, validator_addr) = Address::from_bech32m("sum1qgpqyqszqgpqyqszqgpqyqszqgpqyqszg7k454").unwrap();
        let (_, treasury_addr) = Address::from_bech32m("sumc1qszqgpqyqszqgpqyqszqgpqyqszqgpqypa49fy").unwrap();

        let mut miner = get_account_local(&miner_addr, &account_cache, &self.state_manager)?;
        miner.balance = miner.balance.checked_add(total_miner_payout)
            .ok_or_else(|| ConsensusError::Validation("Miner balance overflow".to_string()))?;
        account_cache.insert(miner_addr, miner);

        let mut validator = get_account_local(&validator_addr, &account_cache, &self.state_manager)?;
        validator.balance = validator.balance.checked_add(total_validator_payout)
            .ok_or_else(|| ConsensusError::Validation("Validator balance overflow".to_string()))?;
        account_cache.insert(validator_addr, validator);

        let mut treasury = get_account_local(&treasury_addr, &account_cache, &self.state_manager)?;
        treasury.balance = treasury.balance.checked_add(total_treasury_payout)
            .ok_or_else(|| ConsensusError::Validation("Treasury balance overflow".to_string()))?;
        account_cache.insert(treasury_addr, treasury);

        Self::compute_state_root_from_updates(parent_state_root, &account_cache)
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
        if block.header.version > 1 || block.header.prev_block_hash != Hash::zero() {
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

        // 5. Validate state root commitment
        let parent_state_root = if block.header.prev_block_hash == Hash::zero() {
            Hash::zero()
        } else {
            let parent_header = self.storage.get_block_header(&block.header.prev_block_hash)?
                .ok_or_else(|| ConsensusError::Validation(format!("Parent block header not found for hash {:?}", block.header.prev_block_hash)))?;
            parent_header.state_root
        };

        let calculated_root = self.calculate_state_root(parent_state_root, block)?;
        if calculated_root != block.header.state_root {
            return Err(ConsensusError::Validation(format!(
                "State root mismatch: expected {:?}, got {:?}",
                block.header.state_root, calculated_root
            )));
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
    use aruna_primitives::{Address, Nonce, TransactionPayload, Difficulty};
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

    #[test]
    fn test_state_root_commitment_validation() {
        let path = temp_db_path();
        {
            let storage = Storage::open(&path).unwrap();
            let state_manager = StateManager::new(storage.clone());
            let engine = ConsensusEngine::new(state_manager.clone(), storage.clone());

            let keypair = Ed25519Keypair::generate();
            let pubkey = keypair.public_key_bytes();
            let pkh = derive_pubkey_hash(&pubkey);
            let sender = Address::from_pubkey_hash(pkh);
            let recipient = Address::from_pubkey_hash([0x22; 20]);

            state_manager.put_account(&sender, &aruna_state::Account::new(10000, Nonce(0))).unwrap();

            // Set up a block with 1 valid transaction
            let payload = TransactionPayload {
                nonce: Nonce(1),
                sender,
                recipient,
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

            let block_body = BlockBody {
                transactions: vec![tx],
                validator_metadata: vec![],
                ecosystem_metadata: vec![],
            };
            let merkle_root = ConsensusEngine::calculate_merkle_root(&block_body.transactions).unwrap();

            let header = BlockHeader {
                version: 1,
                prev_block_hash: Hash::zero(),
                merkle_root,
                state_root: Hash::zero(),
                timestamp: 123456789,
                difficulty: Difficulty(1),
                nonce: 0,
                validator_root: Hash::zero(),
                treasury_root: Hash::zero(),
            };

            let mut block = Block { header, body: block_body };

            // Calculate the correct state root
            let calculated_root = engine.calculate_state_root(Hash::zero(), &block).unwrap();
            
            // If the header has Hash::zero(), it should fail validation because of state root mismatch
            let res_fail = engine.validate_block(&block);
            assert!(matches!(res_fail, Err(ConsensusError::Validation(ref m)) if m.contains("State root mismatch")));

            // If we set the correct state root, it should pass validation
            block.header.state_root = calculated_root;
            let res_success = engine.validate_block(&block);
            assert!(res_success.is_ok());
        }
        let _ = std::fs::remove_dir_all(&path);
    }
}
