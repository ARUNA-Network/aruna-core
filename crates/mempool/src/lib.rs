//! Transaction mempool for the ARUNA Network.
//! Handles transaction buffering, cryptographic and state admission validations, Replace-By-Fee (RBF), and fee-density eviction.

use aruna_primitives::{Address, Hash, TransactionEnvelope, SignatureType};
use aruna_storage::Storage;
use aruna_crypto::{derive_pubkey_hash, Ed25519Verifier};
use thiserror::Error;
use std::collections::HashMap;
use std::sync::RwLock;

/// Error type representing mempool admission and validation failures.
#[derive(Error, Debug)]
pub enum MempoolError {
    #[error("Mempool is at maximum capacity")]
    MempoolFull,
    #[error("Transaction signature verification failed: {0}")]
    InvalidSignature(String),
    #[error("Transaction nonce {got} is too low; on-chain nonce is {expected}")]
    NonceTooLow { expected: u64, got: u64 },
    #[error("Sender account not found on-chain")]
    AccountNotFound,
    #[error("Insufficient balance for sender: available {available} micro-ARU, required {required} micro-ARU")]
    InsufficientBalance { available: u64, required: u64 },
    #[error("Transaction fee {got} is below the minimum fee floor of {required} micro-ARU")]
    FeeTooLow { required: u64, got: u64 },
    #[error("Duplicate nonce {nonce} detected; RBF requires fee of at least {required_fee} micro-ARU (offered {existing_fee})")]
    DuplicateNonce { nonce: u64, existing_fee: u64, required_fee: u64 },
    #[error("Invalid public key length; expected 32 bytes")]
    InvalidPublicKeyLength,
    #[error("Invalid signature length; expected 64 bytes")]
    InvalidSignatureLength,
    #[error("Address mismatch: transaction sender does not match public key hash")]
    AddressMismatch,
    #[error("Unsupported signature type")]
    UnsupportedSignatureType,
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Database read error: {0}")]
    Database(String),
}

/// A thread-safe transaction mempool queue.
pub struct Mempool {
    transactions: RwLock<HashMap<Hash, TransactionEnvelope>>,
    max_capacity: usize,
}

impl Mempool {
    /// Create a new transaction mempool with the specified capacity limit.
    pub fn new(max_capacity: usize) -> Self {
        Self {
            transactions: RwLock::new(HashMap::new()),
            max_capacity,
        }
    }

    /// Validates a transaction envelope and attempts to insert it into the mempool queue.
    /// Handles signature verification, fee floor verification, nonce sequencing, cumulative balance checks, and RBF.
    pub fn add_transaction(&self, tx: TransactionEnvelope, storage: &Storage) -> Result<Hash, MempoolError> {
        // 1. Calculate transaction size and verify fee floor (10 micro-ARU/byte)
        let tx_bytes = aruna_primitives::serialize(&tx)
            .map_err(|e| MempoolError::Serialization(e.to_string()))?;
        let tx_size = tx_bytes.len();
        
        let min_fee = (tx_size as u64) * 10;
        if tx.payload.fee < min_fee {
            return Err(MempoolError::FeeTooLow {
                required: min_fee,
                got: tx.payload.fee,
            });
        }

        // 2. Cryptographic Signature Verification
        match tx.signature_type {
            SignatureType::Ed25519 => {
                if tx.public_key.len() != 32 {
                    return Err(MempoolError::InvalidPublicKeyLength);
                }
                if tx.signature.len() != 64 {
                    return Err(MempoolError::InvalidSignatureLength);
                }

                // Verify public key matches sender address
                let pkh = derive_pubkey_hash(&tx.public_key);
                let derived_sender = Address::from_pubkey_hash(pkh);
                if derived_sender != tx.payload.sender {
                    return Err(MempoolError::AddressMismatch);
                }

                // Verify Ed25519 signature
                let mut pubkey_array = [0u8; 32];
                pubkey_array.copy_from_slice(&tx.public_key);
                
                let mut sig_array = [0u8; 64];
                sig_array.copy_from_slice(&tx.signature);

                let payload_bytes = aruna_primitives::serialize(&tx.payload)
                    .map_err(|e| MempoolError::Serialization(e.to_string()))?;

                Ed25519Verifier::verify(&pubkey_array, &payload_bytes, &sig_array)
                    .map_err(|e| MempoolError::InvalidSignature(e.to_string()))?;
            }
            SignatureType::Secp256k1 => {
                return Err(MempoolError::UnsupportedSignatureType);
            }
        }

        // 3. Retrieve Sender Account State from Database
        let sender_addr = tx.payload.sender;
        let account_opt = storage.get_account(&sender_addr)
            .map_err(|e| MempoolError::Database(e.to_string()))?;
        
        let (on_chain_balance, on_chain_nonce) = match account_opt {
            Some((balance, nonce_val, _, _)) => (balance, nonce_val),
            None => return Err(MempoolError::AccountNotFound),
        };

        // 4. Nonce Verification (must be strictly greater than current on-chain nonce)
        if tx.payload.nonce.0 <= on_chain_nonce {
            return Err(MempoolError::NonceTooLow {
                expected: on_chain_nonce + 1,
                got: tx.payload.nonce.0,
            });
        }

        // 5. Lock mempool map for atomic writes
        let mut txs = self.transactions.write().unwrap();
        let tx_hash = aruna_crypto::blake3_hash(&tx_bytes);

        // 6. Replace-By-Fee (RBF) & Duplicate Nonce Check
        let mut replaced = false;
        let mut to_replace_hash = None;

        for (hash, pending_tx) in txs.iter() {
            if pending_tx.payload.sender == sender_addr && pending_tx.payload.nonce == tx.payload.nonce {
                // RBF rules: fee must increase by at least 10%
                let old_fee = pending_tx.payload.fee;
                let min_new_fee = old_fee + (old_fee / 10);
                if tx.payload.fee >= min_new_fee {
                    to_replace_hash = Some(*hash);
                    replaced = true;
                    break;
                } else {
                    return Err(MempoolError::DuplicateNonce {
                        nonce: tx.payload.nonce.0,
                        existing_fee: old_fee,
                        required_fee: min_new_fee,
                    });
                }
            }
        }

        if let Some(hash) = to_replace_hash {
            txs.remove(&hash);
        }

        // 7. Cumulative Balance Verification
        // The sender's on-chain balance must cover the new transaction AND all other pending transactions from the same sender.
        let mut pending_total = tx.payload.amount + tx.payload.fee;
        for pending_tx in txs.values() {
            if pending_tx.payload.sender == sender_addr {
                pending_total += pending_tx.payload.amount + pending_tx.payload.fee;
            }
        }

        if on_chain_balance < pending_total {
            return Err(MempoolError::InsufficientBalance {
                available: on_chain_balance,
                required: pending_total,
            });
        }

        // 8. Capacity & Eviction Policy (only evaluated if not replacing an existing transaction)
        if !replaced && txs.len() >= self.max_capacity {
            // Locate the transaction with the lowest fee density (fee / serialized size)
            let mut lowest_hash = None;
            let mut lowest_density = f64::MAX;

            for (hash, pending_tx) in txs.iter() {
                if let Ok(p_bytes) = aruna_primitives::serialize(pending_tx) {
                    let density = (pending_tx.payload.fee as f64) / (p_bytes.len() as f64);
                    if density < lowest_density {
                        lowest_density = density;
                        lowest_hash = Some(*hash);
                    }
                }
            }

            let new_density = (tx.payload.fee as f64) / (tx_size as f64);
            if let Some(low_hash) = lowest_hash {
                if new_density > lowest_density {
                    // Evict the lowest-density transaction to make room
                    txs.remove(&low_hash);
                } else {
                    return Err(MempoolError::MempoolFull);
                }
            } else {
                return Err(MempoolError::MempoolFull);
            }
        }

        // 9. Insert transaction envelope
        txs.insert(tx_hash, tx);

        Ok(tx_hash)
    }

    /// Retrieve the highest-priority pending transactions up to the specified limit, sorted by fee density.
    pub fn get_pending_transactions(&self, limit: usize) -> Vec<TransactionEnvelope> {
        let txs = self.transactions.read().unwrap();
        let mut sorted_txs: Vec<&TransactionEnvelope> = txs.values().collect();

        sorted_txs.sort_by(|a, b| {
            if a.payload.sender == b.payload.sender {
                a.payload.nonce.cmp(&b.payload.nonce)
            } else {
                let a_size = aruna_primitives::serialize(a).map(|b| b.len()).unwrap_or(1);
                let b_size = aruna_primitives::serialize(b).map(|b| b.len()).unwrap_or(1);
                
                let a_density = (a.payload.fee as f64) / (a_size as f64);
                let b_density = (b.payload.fee as f64) / (b_size as f64);

                b_density.partial_cmp(&a_density).unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        sorted_txs.into_iter().take(limit).cloned().collect()
    }

    /// Remove a list of transactions (by their hashes) from the mempool (typically on block commit).
    pub fn remove_transactions(&self, hashes: &[Hash]) {
        let mut txs = self.transactions.write().unwrap();
        for hash in hashes {
            txs.remove(hash);
        }
    }

    /// Returns the number of transactions currently waiting in the mempool.
    pub fn len(&self) -> usize {
        self.transactions.read().unwrap().len()
    }

    /// Returns the maximum capacity configuration of this mempool.
    pub fn capacity(&self) -> usize {
        self.max_capacity
    }

    /// Returns whether the mempool contains a transaction with the given hash.
    pub fn contains(&self, hash: &Hash) -> bool {
        self.transactions.read().unwrap().contains_key(hash)
    }

    /// Returns the transaction envelope if it exists in the mempool.
    pub fn get_transaction(&self, hash: &Hash) -> Option<TransactionEnvelope> {
        self.transactions.read().unwrap().get(hash).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aruna_primitives::{TransactionPayload, Nonce};
    use aruna_crypto::Ed25519Keypair;
    use std::path::PathBuf;

    fn temp_db_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("aruna_mempool_test_{}", time));
        path
    }

    #[test]
    fn test_mempool_admission_and_validation() {
        let db_path = temp_db_path();
        let storage = Storage::open(&db_path).unwrap();
        
        // Generate sender keypair
        let keypair = Ed25519Keypair::generate();
        let pubkey = keypair.public_key_bytes();
        let sender = Address::from_pubkey_hash(derive_pubkey_hash(&pubkey));
        let recipient = Address::from_pubkey_hash([0xde; 20]);

        // Initialize sender balance and nonce in database
        storage.put_account(&sender, 10_000_000, 0, &Hash::zero(), &Hash::zero()).unwrap();

        let mempool = Mempool::new(10);

        // 1. Valid transaction insertion
        let payload = TransactionPayload {
            nonce: Nonce(1),
            sender,
            recipient,
            amount: 100_000,
            fee: 5_000, // ~500 bytes max, 5000 is plenty
            gas_limit: 0,
            gas_price: 0,
            data: vec![],
        };
        let payload_bytes = aruna_primitives::serialize(&payload).unwrap();
        let signature = keypair.sign(&payload_bytes);
        
        let tx = TransactionEnvelope {
            payload: payload.clone(),
            signature_type: SignatureType::Ed25519,
            signature: signature.to_vec(),
            public_key: pubkey.to_vec(),
        };

        let result = mempool.add_transaction(tx.clone(), &storage);
        assert!(result.is_ok());
        assert_eq!(mempool.len(), 1);
        assert!(mempool.contains(&result.unwrap()));

        // 2. Invalid Signature Rejection
        let mut invalid_tx = tx.clone();
        if !invalid_tx.signature.is_empty() {
            invalid_tx.signature[0] ^= 0xFF; // Corrupt signature
        }
        let res_sig = mempool.add_transaction(invalid_tx, &storage);
        assert!(matches!(res_sig, Err(MempoolError::InvalidSignature(_))));

        // 3. Nonce Too Low Rejection
        let payload_low_nonce = TransactionPayload {
            nonce: Nonce(0), // on-chain is 0, so 0 is too low
            ..payload.clone()
        };
        let low_nonce_bytes = aruna_primitives::serialize(&payload_low_nonce).unwrap();
        let low_nonce_sig = keypair.sign(&low_nonce_bytes);
        let tx_low_nonce = TransactionEnvelope {
            payload: payload_low_nonce,
            signature: low_nonce_sig.to_vec(),
            ..tx.clone()
        };
        let res_nonce = mempool.add_transaction(tx_low_nonce, &storage);
        assert!(matches!(res_nonce, Err(MempoolError::NonceTooLow { .. })));

        // 4. Insufficient Balance Rejection
        let payload_high_amount = TransactionPayload {
            nonce: Nonce(2),
            amount: 20_000_000, // Exceeds 10,000_000 on-chain balance
            ..payload.clone()
        };
        let high_amount_bytes = aruna_primitives::serialize(&payload_high_amount).unwrap();
        let high_amount_sig = keypair.sign(&high_amount_bytes);
        let tx_high_amount = TransactionEnvelope {
            payload: payload_high_amount,
            signature: high_amount_sig.to_vec(),
            ..tx.clone()
        };
        let res_balance = mempool.add_transaction(tx_high_amount, &storage);
        assert!(matches!(res_balance, Err(MempoolError::InsufficientBalance { .. })));

        // Clean up DB directory
        let _ = std::fs::remove_dir_all(db_path);
    }

    #[test]
    fn test_mempool_replace_by_fee() {
        let db_path = temp_db_path();
        let storage = Storage::open(&db_path).unwrap();
        
        let keypair = Ed25519Keypair::generate();
        let pubkey = keypair.public_key_bytes();
        let sender = Address::from_pubkey_hash(derive_pubkey_hash(&pubkey));
        let recipient = Address::from_pubkey_hash([0xde; 20]);
        storage.put_account(&sender, 10_000_000, 0, &Hash::zero(), &Hash::zero()).unwrap();

        let mempool = Mempool::new(10);

        // First transaction with fee = 5000
        let payload1 = TransactionPayload {
            nonce: Nonce(1),
            sender,
            recipient,
            amount: 100_000,
            fee: 5_000,
            gas_limit: 0,
            gas_price: 0,
            data: vec![],
        };
        let p1_bytes = aruna_primitives::serialize(&payload1).unwrap();
        let sig1 = keypair.sign(&p1_bytes);
        let tx1 = TransactionEnvelope {
            payload: payload1,
            signature_type: SignatureType::Ed25519,
            signature: sig1.to_vec(),
            public_key: pubkey.to_vec(),
        };
        let hash1 = mempool.add_transaction(tx1.clone(), &storage).unwrap();

        // Second transaction with same nonce, but insufficient fee increase (5200 is only 4% increase)
        let payload2_bad = TransactionPayload {
            fee: 5_200,
            ..tx1.payload.clone()
        };
        let p2_bad_bytes = aruna_primitives::serialize(&payload2_bad).unwrap();
        let sig2_bad = keypair.sign(&p2_bad_bytes);
        let tx2_bad = TransactionEnvelope {
            payload: payload2_bad,
            signature: sig2_bad.to_vec(),
            signature_type: SignatureType::Ed25519,
            public_key: pubkey.to_vec(),
        };
        let res_rbf_bad = mempool.add_transaction(tx2_bad, &storage);
        assert!(matches!(res_rbf_bad, Err(MempoolError::DuplicateNonce { .. })));

        // Third transaction with same nonce, and sufficient fee increase (6000 is 20% increase)
        let payload2_good = TransactionPayload {
            fee: 6_000,
            ..tx1.payload.clone()
        };
        let p2_good_bytes = aruna_primitives::serialize(&payload2_good).unwrap();
        let sig2_good = keypair.sign(&p2_good_bytes);
        let tx2_good = TransactionEnvelope {
            payload: payload2_good,
            signature: sig2_good.to_vec(),
            signature_type: SignatureType::Ed25519,
            public_key: pubkey.to_vec(),
        };
        let hash2 = mempool.add_transaction(tx2_good, &storage).unwrap();

        // The old transaction must be evicted, and the new one accepted
        assert_eq!(mempool.len(), 1);
        assert!(!mempool.contains(&hash1));
        assert!(mempool.contains(&hash2));

        let _ = std::fs::remove_dir_all(db_path);
    }
}
