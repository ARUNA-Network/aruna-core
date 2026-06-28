//! Consensus validation engine for the ARUNA Network.
//! Conforms to specifications defined in docs/protocol/consensus.md, block.md, and transaction.md.

use aruna_primitives::{Block, BlockHeader, BlockBody, TransactionEnvelope, Hash, serialize, SignatureType, Address};
use std::time::SystemTime;
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
    /// Address that receives the PoW miner block reward (70% of block reward + 70% of fees).
    /// Must be sourced from node configuration — never hardcoded.
    pub miner_reward_addr: Address,
    /// Address that receives the PoS validator block reward (25% of block reward + 25% of fees).
    pub validator_reward_addr: Address,
    /// Address that receives the treasury allocation (5% of block reward + remaining fees).
    pub treasury_reward_addr: Address,
}

impl ConsensusEngine {
    /// Create a new ConsensusEngine instance.
    ///
    /// # Arguments
    /// * `state_manager` — ledger state manager
    /// * `storage` — RocksDB storage backend
    /// * `miner_reward_addr` — address receiving 70% PoW block reward
    /// * `validator_reward_addr` — address receiving 25% PoS block reward
    /// * `treasury_reward_addr` — address receiving 5% treasury allocation
    pub fn new(
        state_manager: StateManager,
        storage: Storage,
        miner_reward_addr: Address,
        validator_reward_addr: Address,
        treasury_reward_addr: Address,
    ) -> Self {
        Self {
            state_manager,
            storage,
            miner_reward_addr,
            validator_reward_addr,
            treasury_reward_addr,
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

    /// Calculate accumulated work from compact difficulty (nBits).
    /// Returns the difficulty value as the work unit, enabling proper most-work fork choice.
    /// Sumatera Testnet uses constant difficulty (504_381_424), so each block contributes
    /// exactly that many units of cumulative work to the chain.
    pub fn block_work(difficulty: aruna_primitives::Difficulty) -> u128 {
        difficulty.0 as u128
    }

    /// Appled block transitions and rewards, writing them to StorageBatch and returning computed state root.
    pub fn apply_block_state(&self, block: &Block, batch: &mut StorageBatch) -> Result<Hash, ConsensusError> {
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

        // 1. Execute all transactions
        for (tx_idx, tx) in block.body.transactions.iter().enumerate() {
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

            // Index transaction
            let tx_bytes = serialize(tx)
                .map_err(|e| ConsensusError::Database(StorageError::Format(e.to_string())))?;
            let tx_hash = aruna_crypto::blake3_hash(&tx_bytes);
            let header_bytes = serialize(&block.header)
                .map_err(|e| ConsensusError::Database(StorageError::Format(e.to_string())))?;
            let block_hash = aruna_crypto::blake3_hash(&header_bytes);
            batch.put_tx_index(&tx_hash, &block_hash, tx_idx as u32);
        }

        // 2. Distribute block rewards
        let new_height = if block.header.prev_block_hash == Hash::zero() {
            0
        } else {
            let parent_height = self.storage.get_block_height_by_hash(&block.header.prev_block_hash)?
                .ok_or_else(|| ConsensusError::Validation(format!("Parent block height missing for hash {:?}", block.header.prev_block_hash)))?;
            parent_height + 1
        };

        let total_fees: u64 = block.body.transactions.iter().map(|tx| tx.payload.fee).sum();
        let (miner_reward, validator_reward, treasury_reward) = Self::calculate_reward(new_height);

        let total_miner_payout = miner_reward + (total_fees * 70 / 100);
        let total_validator_payout = validator_reward + (total_fees * 25 / 100);
        
        let total_pool = miner_reward + validator_reward + treasury_reward + total_fees;
        let total_treasury_payout = total_pool - total_miner_payout - total_validator_payout;

        let miner_addr = self.miner_reward_addr;
        let validator_addr = self.validator_reward_addr;
        let treasury_addr = self.treasury_reward_addr;

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

        // Write modified accounts to batch
        for (addr, acc) in &account_cache {
            batch.put_account(addr, acc.balance, acc.nonce.0, &acc.code_hash, &acc.storage_root);
        }

        // Calculate expected state root
        let parent_state_root = if block.header.prev_block_hash == Hash::zero() {
            Hash::zero()
        } else {
            let parent_header = self.storage.get_block_header(&block.header.prev_block_hash)?
                .ok_or_else(|| ConsensusError::Validation(format!("Parent block header missing for hash {:?}", block.header.prev_block_hash)))?;
            parent_header.state_root
        };

        Self::compute_state_root_from_updates(parent_state_root, &account_cache)
    }

    /// Reverts the state transitions of a block, updating account balances and nonces backwards.
    pub fn rollback_block(&self, block: &Block, batch: &mut StorageBatch) -> Result<(), ConsensusError> {
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

        // 1. Revert all transactions in the block in REVERSE order
        for tx in block.body.transactions.iter().rev() {
            let sender_addr = tx.payload.sender;
            let recipient_addr = tx.payload.recipient;

            let mut sender = get_account_local(&sender_addr, &account_cache, &self.state_manager)?;
            let mut recipient = get_account_local(&recipient_addr, &account_cache, &self.state_manager)?;

            let total_required = tx.payload.amount.checked_add(tx.payload.fee)
                .ok_or_else(|| ConsensusError::Validation("Transaction amount/fee overflow".to_string()))?;

            sender.balance = sender.balance.checked_add(total_required)
                .ok_or_else(|| ConsensusError::Validation("Rollback sender balance overflow".to_string()))?;
            sender.nonce = aruna_primitives::Nonce(tx.payload.nonce.0.saturating_sub(1));

            recipient.balance = recipient.balance.checked_sub(tx.payload.amount)
                .ok_or_else(|| ConsensusError::Validation("Rollback recipient balance underflow".to_string()))?;

            account_cache.insert(sender_addr, sender);
            account_cache.insert(recipient_addr, recipient);
        }

        // 2. Revert rewards
        let header_bytes = serialize(&block.header)
            .map_err(|e| ConsensusError::Database(StorageError::Format(e.to_string())))?;
        let block_hash = aruna_crypto::blake3_hash(&header_bytes);

        let block_height = self.storage.get_block_height_by_hash(&block_hash)?
            .ok_or_else(|| ConsensusError::Validation("Block height index missing during rollback".to_string()))?;

        let total_fees: u64 = block.body.transactions.iter().map(|tx| tx.payload.fee).sum();
        let (miner_reward, validator_reward, treasury_reward) = Self::calculate_reward(block_height);

        let total_miner_payout = miner_reward + (total_fees * 70 / 100);
        let total_validator_payout = validator_reward + (total_fees * 25 / 100);
        
        let total_pool = miner_reward + validator_reward + treasury_reward + total_fees;
        let total_treasury_payout = total_pool - total_miner_payout - total_validator_payout;

        let miner_addr = self.miner_reward_addr;
        let validator_addr = self.validator_reward_addr;
        let treasury_addr = self.treasury_reward_addr;

        let mut miner = get_account_local(&miner_addr, &account_cache, &self.state_manager)?;
        miner.balance = miner.balance.checked_sub(total_miner_payout)
            .ok_or_else(|| ConsensusError::Validation("Rollback miner balance underflow".to_string()))?;
        account_cache.insert(miner_addr, miner);

        let mut validator = get_account_local(&validator_addr, &account_cache, &self.state_manager)?;
        validator.balance = validator.balance.checked_sub(total_validator_payout)
            .ok_or_else(|| ConsensusError::Validation("Rollback validator balance underflow".to_string()))?;
        account_cache.insert(validator_addr, validator);

        let mut treasury = get_account_local(&treasury_addr, &account_cache, &self.state_manager)?;
        treasury.balance = treasury.balance.checked_sub(total_treasury_payout)
            .ok_or_else(|| ConsensusError::Validation("Rollback treasury balance underflow".to_string()))?;
        account_cache.insert(treasury_addr, treasury);

        // 3. Write updates to batch
        for (addr, acc) in account_cache {
            batch.put_account(&addr, acc.balance, acc.nonce.0, &acc.code_hash, &acc.storage_root);
        }

        Ok(())
    }

    /// Commits a block to the RocksDB database, resolving forks using the Fork Choice Rule (FCR).
    pub fn commit_block(&self, block: &Block) -> Result<Hash, ConsensusError> {
        let header_bytes = serialize(&block.header)
            .map_err(|e| ConsensusError::Database(StorageError::Format(e.to_string())))?;
        let block_hash = aruna_crypto::blake3_hash(&header_bytes);

        // If block already exists in storage, return early
        if self.storage.get_block_header(&block_hash)?.is_some() {
            println!("Block {:?} already exists in database. Ignoring duplicate.", block_hash);
            return Ok(block_hash);
        }

        // Retrieve parent's cumulative difficulty
        let parent_cum_diff = if block.header.prev_block_hash == Hash::zero() {
            0_u128
        } else {
            self.storage.get_cumulative_difficulty(&block.header.prev_block_hash)?
                .ok_or_else(|| ConsensusError::Validation(format!(
                    "Parent cumulative difficulty missing for hash {:?}", block.header.prev_block_hash
                )))?
        };

        let new_cum_diff = parent_cum_diff + Self::block_work(block.header.difficulty);

        // Fetch best block's cumulative difficulty
        let best_hash = self.storage.get_best_block()?.unwrap_or(Hash::zero());
        let best_cum_diff = if best_hash == Hash::zero() {
            0_u128
        } else {
            self.storage.get_cumulative_difficulty(&best_hash)?
                .unwrap_or(0)
        };

        // Determine if this block extends/replaces the canonical tip (higher work)
        if new_cum_diff > best_cum_diff {
            if block.header.prev_block_hash == best_hash {
                // Case 1: Normal extension of canonical tip (Fast path)
                self.validate_block(block)?;

                let mut batch = StorageBatch::new();
                let state_root = self.apply_block_state(block, &mut batch)?;
                if state_root != block.header.state_root {
                    return Err(ConsensusError::Validation(format!(
                        "State root mismatch: expected {:?}, got {:?}",
                        block.header.state_root, state_root
                    )));
                }

                let current_height = self.storage.get_chain_height()?.unwrap_or(0);
                let new_height = if block.header.prev_block_hash == Hash::zero() { 0 } else { current_height + 1 };

                batch.put_block_height_map(new_height, &block_hash);
                batch.put_block_height_by_hash(&block_hash, new_height);
                batch.put_cumulative_difficulty(&block_hash, new_cum_diff);

                self.storage.write_batch(batch)?;

                self.storage.put_block_header(&block_hash, &block.header)?;
                self.storage.put_block_body(&block_hash, &block.body)?;
                self.storage.put_best_block(&block_hash)?;
                self.storage.put_chain_height(new_height)?;
            } else {
                // Case 2: Chain Reorganization (FCR switch to heavier fork)
                println!("Fork choice rule: heavier chain fork detected. Starting reorganization...");

                let mut disconnect_path = Vec::new();
                let mut connect_path = Vec::new();

                let mut curr_disconnect = best_hash;
                let mut curr_connect = block.header.prev_block_hash;

                let mut height_disconnect = self.storage.get_block_height_by_hash(&curr_disconnect)?
                    .ok_or_else(|| ConsensusError::Validation("Tip height missing".to_string()))?;
                let mut height_connect = self.storage.get_block_height_by_hash(&curr_connect)?
                    .ok_or_else(|| ConsensusError::Validation("Fork parent height missing".to_string()))?;

                while height_disconnect > height_connect {
                    let header = self.storage.get_block_header(&curr_disconnect)?
                        .ok_or_else(|| ConsensusError::Validation("Header missing".to_string()))?;
                    disconnect_path.push((curr_disconnect, header));
                    curr_disconnect = disconnect_path.last().unwrap().1.prev_block_hash;
                    height_disconnect -= 1;
                }

                while height_connect > height_disconnect {
                    let header = self.storage.get_block_header(&curr_connect)?
                        .ok_or_else(|| ConsensusError::Validation("Header missing".to_string()))?;
                    connect_path.push((curr_connect, header));
                    curr_connect = connect_path.last().unwrap().1.prev_block_hash;
                    height_connect -= 1;
                }

                while curr_disconnect != curr_connect && curr_disconnect != Hash::zero() {
                    let header_d = self.storage.get_block_header(&curr_disconnect)?
                        .ok_or_else(|| ConsensusError::Validation("Header missing".to_string()))?;
                    disconnect_path.push((curr_disconnect, header_d));
                    curr_disconnect = disconnect_path.last().unwrap().1.prev_block_hash;

                    let header_c = self.storage.get_block_header(&curr_connect)?
                        .ok_or_else(|| ConsensusError::Validation("Header missing".to_string()))?;
                    connect_path.push((curr_connect, header_c));
                    curr_connect = connect_path.last().unwrap().1.prev_block_hash;
                }

                let mut connect_blocks = Vec::new();
                for (hash, header) in connect_path.into_iter().rev() {
                    let body = self.storage.get_block_body(&hash)?
                        .ok_or_else(|| ConsensusError::Validation("Body missing".to_string()))?;
                    connect_blocks.push(Block { header, body });
                }
                connect_blocks.push(block.clone());

                self.storage.put_block_header(&block_hash, &block.header)?;
                self.storage.put_block_body(&block_hash, &block.body)?;
                self.storage.put_cumulative_difficulty(&block_hash, new_cum_diff)?;

                // Perform state rollback block-by-block to ensure correct account state reads
                for (hash, header) in &disconnect_path {
                    let body = self.storage.get_block_body(hash)?
                        .ok_or_else(|| ConsensusError::Validation("Body missing".to_string()))?;
                    let mut rollback_batch = StorageBatch::new();
                    self.rollback_block(&Block { header: header.clone(), body }, &mut rollback_batch)?;
                    self.storage.write_batch(rollback_batch)?;
                }

                let mut applied_blocks = Vec::new();
                let mut success = true;
                let mut validation_err = None;

                for reorg_block in &connect_blocks {
                    let r_bytes = serialize(&reorg_block.header).unwrap();
                    let r_hash = aruna_crypto::blake3_hash(&r_bytes);

                    if let Err(e) = self.validate_block(reorg_block) {
                        success = false;
                        validation_err = Some(e);
                        break;
                    }

                    let mut apply_batch = StorageBatch::new();
                    match self.apply_block_state(reorg_block, &mut apply_batch) {
                        Ok(state_root) => {
                            if state_root != reorg_block.header.state_root {
                                success = false;
                                validation_err = Some(ConsensusError::Validation(format!(
                                    "State root mismatch on fork block: expected {:?}, got {:?}",
                                    reorg_block.header.state_root, state_root
                                )));
                                break;
                            }

                            let parent_height = if reorg_block.header.prev_block_hash == Hash::zero() {
                                0
                            } else {
                                self.storage.get_block_height_by_hash(&reorg_block.header.prev_block_hash)?.unwrap_or(0)
                            };
                            let r_height = if reorg_block.header.prev_block_hash == Hash::zero() { 0 } else { parent_height + 1 };

                            apply_batch.put_block_height_map(r_height, &r_hash);
                            apply_batch.put_block_height_by_hash(&r_hash, r_height);

                            let r_parent_cum_diff = if reorg_block.header.prev_block_hash == Hash::zero() {
                                0
                            } else {
                                self.storage.get_cumulative_difficulty(&reorg_block.header.prev_block_hash)?.unwrap_or(0)
                            };
                            let r_cum_diff = r_parent_cum_diff + Self::block_work(reorg_block.header.difficulty);
                            apply_batch.put_cumulative_difficulty(&r_hash, r_cum_diff);

                            self.storage.write_batch(apply_batch)?;
                            applied_blocks.push((r_hash, reorg_block.clone(), r_height));
                        }
                        Err(e) => {
                            success = false;
                            validation_err = Some(e);
                            break;
                        }
                    }
                }

                if success {
                    let (final_hash, _, final_height) = applied_blocks.last().unwrap();
                    self.storage.put_best_block(final_hash)?;
                    self.storage.put_chain_height(*final_height)?;
                    println!("Reorganization complete! Canonical tip switched to {} at height {}", final_hash, final_height);
                } else {
                    println!("Reorganization failed: {:?}. Restoring original chain state...", validation_err);

                    let mut restore_rollback_batch = StorageBatch::new();
                    for (_, b, _) in applied_blocks.iter().rev() {
                        self.rollback_block(b, &mut restore_rollback_batch)?;
                    }
                    self.storage.write_batch(restore_rollback_batch)?;

                    for (hash, header) in disconnect_path.iter().rev() {
                        let body = self.storage.get_block_body(hash)?
                            .ok_or_else(|| ConsensusError::Validation("Body missing".to_string()))?;
                        let mut restore_apply_batch = StorageBatch::new();
                        self.apply_block_state(&Block { header: header.clone(), body }, &mut restore_apply_batch)?;
                        self.storage.write_batch(restore_apply_batch)?;
                    }

                    return Err(validation_err.unwrap());
                }
            }
        } else {
            // Case 3: Side-chain Storage (lighter branch block)
            println!("Received block on a lighter branch (cumulative difficulty {} <= canonical {}). Storing as side-chain.", new_cum_diff, best_cum_diff);
            
            self.validate_block_header_only(block)?;

            self.storage.put_block_header(&block_hash, &block.header)?;
            self.storage.put_block_body(&block_hash, &block.body)?;
            self.storage.put_cumulative_difficulty(&block_hash, new_cum_diff)?;

            let parent_height = if block.header.prev_block_hash == Hash::zero() {
                0
            } else {
                self.storage.get_block_height_by_hash(&block.header.prev_block_hash)?
                    .unwrap_or(0)
            };
            let side_height = if block.header.prev_block_hash == Hash::zero() { 0 } else { parent_height + 1 };
            self.storage.put_block_height_by_hash(&block_hash, side_height)?;
        }

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

    /// Validates a single transaction's cryptographic signatures only.
    pub fn validate_transaction_signatures_only(&self, tx: &TransactionEnvelope) -> Result<(), ConsensusError> {
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
        Ok(())
    }

    /// Validates a single transaction, including cryptographic signatures and ledger account states.
    pub fn validate_transaction(&self, tx: &TransactionEnvelope) -> Result<(), ConsensusError> {
        self.validate_transaction_signatures_only(tx)?;

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

        let miner_addr = self.miner_reward_addr;
        let validator_addr = self.validator_reward_addr;
        let treasury_addr = self.treasury_reward_addr;

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

    /// Performs validation on block size, parent link, Merkle root and signatures without dry-running ledger state.
    pub fn validate_block_header_only(&self, block: &Block) -> Result<(), ConsensusError> {
        let block_bytes = serialize(block).map_err(|e| ConsensusError::Database(StorageError::Format(e.to_string())))?;
        if block_bytes.len() > 2 * 1024 * 1024 {
            return Err(ConsensusError::BlockSizeExceeded {
                size: block_bytes.len(),
            });
        }

        if block.header.version > 1 || block.header.prev_block_hash != Hash::zero() {
            let parent_header = self.storage.get_block_header(&block.header.prev_block_hash)?
                .ok_or_else(|| ConsensusError::Validation(format!(
                    "Parent block header not found for hash {:?}",
                    block.header.prev_block_hash
                )))?;

            // Timestamp Rule 1: strictly increasing timestamps
            if block.header.timestamp <= parent_header.timestamp {
                return Err(ConsensusError::InvalidTimestamp {
                    timestamp: block.header.timestamp,
                    min_timestamp: parent_header.timestamp + 1,
                });
            }
        }

        // Timestamp Rule 2: Future Limit (max 2 hours / 7200 seconds in the future)
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| ConsensusError::Validation(e.to_string()))?
            .as_secs();
        if block.header.timestamp > now + 7200 {
            return Err(ConsensusError::Validation(format!(
                "Block timestamp {} is too far in the future (limit: {})",
                block.header.timestamp, now + 7200
            )));
        }

        let derived_merkle = Self::calculate_merkle_root(&block.body.transactions)?;
        if block.header.merkle_root != derived_merkle {
            return Err(ConsensusError::InvalidMerkleRoot {
                expected: derived_merkle,
                got: block.header.merkle_root,
            });
        }

        for tx in &block.body.transactions {
            self.validate_transaction_signatures_only(tx)?;

            // Enforce minimum transaction fee floor (10 micro-ARU/byte, min 2280 micro-ARU)
            let tx_bytes = serialize(tx).map_err(|e| ConsensusError::Database(StorageError::Format(e.to_string())))?;
            let min_fee = ((tx_bytes.len() as u64) * 10).max(2280);
            if tx.payload.fee < min_fee {
                return Err(ConsensusError::Validation(format!(
                    "Transaction fee too low: expected at least {}, got {}",
                    min_fee, tx.payload.fee
                )));
            }
        }

        Ok(())
    }

    /// Validates an entire block header, Merkle tree alignments, and transaction list constraints.
    pub fn validate_block(&self, block: &Block) -> Result<(), ConsensusError> {
        self.validate_block_header_only(block)?;

        // Validate state root commitment if parent matches best block (tip)
        let best_hash = self.storage.get_best_block()?.unwrap_or(Hash::zero());
        if block.header.prev_block_hash == best_hash {
            let parent_state_root = if best_hash == Hash::zero() {
                Hash::zero()
            } else {
                let parent_header = self.storage.get_block_header(&best_hash)?
                    .ok_or_else(|| ConsensusError::Validation("Parent block header missing".to_string()))?;
                parent_header.state_root
            };

            let calculated_root = self.calculate_state_root(parent_state_root, block)?;
            if calculated_root != block.header.state_root {
                return Err(ConsensusError::Validation(format!(
                    "State root mismatch: expected {:?}, got {:?}",
                    block.header.state_root, calculated_root
                )));
            }
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
            let engine = ConsensusEngine::new(
                state_manager.clone(),
                storage,
                Address::from_pubkey_hash([0x01; 20]),
                Address::from_pubkey_hash([0x02; 20]),
                Address::from_pubkey_hash([0x03; 20]),
            );

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
            let engine = ConsensusEngine::new(
                state_manager.clone(),
                storage.clone(),
                Address::from_pubkey_hash([0x01; 20]),
                Address::from_pubkey_hash([0x02; 20]),
                Address::from_pubkey_hash([0x03; 20]),
            );

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
                fee: 5000,
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
