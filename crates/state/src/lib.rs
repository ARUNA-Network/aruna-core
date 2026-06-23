//! State management layer for the ARUNA Network ledger.
//! Conforms to state specifications defined in docs/protocol/state.md.

use aruna_primitives::{Address, Hash, Nonce, TransactionEnvelope};
use aruna_storage::{Storage, StorageBatch, StorageError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for ledger state updates and validations.
#[derive(Error, Debug)]
pub enum StateError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Nonce mismatch for address {address:?}: expected {expected:?}, got {got:?}")]
    NonceMismatch {
        address: Address,
        expected: Nonce,
        got: Nonce,
    },
    #[error("Insufficient balance for address {address:?}: has {balance} micro-ARU, requires {required} micro-ARU")]
    InsufficientBalance {
        address: Address,
        balance: u64,
        required: u64,
    },
    #[error("Sender account not found in database: {0:?}")]
    AccountNotFound(Address),
}

/// The state representation of a ledger account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    /// Account balance in micro-ARU.
    pub balance: u64,
    /// Transaction nonce (counter).
    pub nonce: Nonce,
    /// Hash of deployed EVM smart contract bytecode (Hash::zero() if standard account).
    pub code_hash: Hash,
    /// Root hash of the contract internal key-value storage (Hash::zero() if empty).
    pub storage_root: Hash,
}

impl Account {
    /// Create a new account with the specified balance.
    pub fn new(balance: u64, nonce: Nonce) -> Self {
        Self {
            balance,
            nonce,
            code_hash: Hash::zero(),
            storage_root: Hash::zero(),
        }
    }
}

/// Manages database reads/writes and ledger state transition rules.
pub struct StateManager {
    storage: Storage,
}

impl StateManager {
    /// Create a new StateManager wrapping the provided Storage backend.
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    /// Read the Account state of the specified Address.
    pub fn get_account(&self, address: &Address) -> Result<Option<Account>, StateError> {
        match self.storage.get_account(address)? {
            Some((balance, nonce_val, code_hash, storage_root)) => {
                Ok(Some(Account {
                    balance,
                    nonce: Nonce(nonce_val),
                    code_hash,
                    storage_root,
                }))
            }
            None => Ok(None),
        }
    }

    /// Write the Account state of the specified Address directly to database storage.
    pub fn put_account(&self, address: &Address, account: &Account) -> Result<(), StateError> {
        self.storage.put_account(
            address,
            account.balance,
            account.nonce.0,
            &account.code_hash,
            &account.storage_root,
        )?;
        Ok(())
    }

    /// Validates and applies a transaction envelope, adding updates to the write batch.
    pub fn apply_transaction(&self, tx: &TransactionEnvelope, batch: &mut StorageBatch) -> Result<(), StateError> {
        let sender_addr = tx.payload.sender;
        let recipient_addr = tx.payload.recipient;

        // 1. Retrieve and validate Sender account
        let mut sender = self
            .get_account(&sender_addr)?
            .ok_or(StateError::AccountNotFound(sender_addr))?;

        // 2. Validate Nonce (Strict Increment: Tx.nonce must equal Account.nonce + 1)
        let expected_nonce = sender.nonce.increment();
        if tx.payload.nonce != expected_nonce {
            return Err(StateError::NonceMismatch {
                address: sender_addr,
                expected: expected_nonce,
                got: tx.payload.nonce,
            });
        }

        // 3. Validate Balance (Sender must cover Amount + Fee)
        let total_required = tx
            .payload
            .amount
            .checked_add(tx.payload.fee)
            .ok_or_else(|| StateError::InsufficientBalance {
                address: sender_addr,
                balance: sender.balance,
                required: u64::MAX,
            })?;

        if sender.balance < total_required {
            return Err(StateError::InsufficientBalance {
                address: sender_addr,
                balance: sender.balance,
                required: total_required,
            });
        }

        // 4. Retrieve or create Recipient account
        let mut recipient = self.get_account(&recipient_addr)?.unwrap_or_else(|| Account {
            balance: 0,
            nonce: Nonce::zero(),
            code_hash: Hash::zero(),
            storage_root: Hash::zero(),
        });

        // 5. Apply state transitions (fees are deducted per-transaction, distribution occurs block-wide)
        sender.balance -= total_required;
        sender.nonce = tx.payload.nonce;

        recipient.balance = recipient
            .balance
            .checked_add(tx.payload.amount)
            .ok_or_else(|| StateError::InsufficientBalance {
                address: recipient_addr,
                balance: recipient.balance,
                required: tx.payload.amount,
            })?;

        // 6. Append writes to batch
        batch.put_account(
            &sender_addr,
            sender.balance,
            sender.nonce.0,
            &sender.code_hash,
            &sender.storage_root,
        );

        batch.put_account(
            &recipient_addr,
            recipient.balance,
            recipient.nonce.0,
            &recipient.code_hash,
            &recipient.storage_root,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aruna_primitives::{Difficulty, SignatureType, TransactionPayload};

    fn temp_db_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("aruna_state_test_{}", time));
        path
    }

    #[test]
    fn test_apply_transaction_success() {
        let path = temp_db_path();
        {
            let storage = Storage::open(&path).unwrap();
            let state = StateManager::new(storage.clone());

            let sender_addr = Address::from_pubkey_hash([0x11; 20]);
            let recipient_addr = Address::from_pubkey_hash([0x22; 20]);

            let sender_acc = Account::new(1000, Nonce(0));
            state.put_account(&sender_addr, &sender_acc).unwrap();

            let tx = TransactionEnvelope {
                payload: TransactionPayload {
                    nonce: Nonce(1),
                    sender: sender_addr,
                    recipient: recipient_addr,
                    amount: 400,
                    fee: 50,
                    gas_limit: 0,
                    gas_price: 0,
                    data: vec![],
                },
                signature_type: SignatureType::Ed25519,
                signature: vec![0; 64],
                public_key: vec![0; 32],
            };

            let mut batch = StorageBatch::new();
            state.apply_transaction(&tx, &mut batch).unwrap();
            storage.write_batch(batch).unwrap();

            // Verify sender balance deducted and nonce incremented
            let sender_post = state.get_account(&sender_addr).unwrap().unwrap();
            assert_eq!(sender_post.balance, 550); // 1000 - 400 - 50
            assert_eq!(sender_post.nonce, Nonce(1));

            // Verify recipient created and balance credited
            let recipient_post = state.get_account(&recipient_addr).unwrap().unwrap();
            assert_eq!(recipient_post.balance, 400);
            assert_eq!(recipient_post.nonce, Nonce(0));
        }
        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn test_apply_transaction_nonce_mismatch() {
        let path = temp_db_path();
        {
            let storage = Storage::open(&path).unwrap();
            let state = StateManager::new(storage);

            let sender_addr = Address::from_pubkey_hash([0x11; 20]);
            let recipient_addr = Address::from_pubkey_hash([0x22; 20]);

            let sender_acc = Account::new(1000, Nonce(0));
            state.put_account(&sender_addr, &sender_acc).unwrap();

            let tx = TransactionEnvelope {
                payload: TransactionPayload {
                    nonce: Nonce(5), // Expected Nonce(1)
                    sender: sender_addr,
                    recipient: recipient_addr,
                    amount: 400,
                    fee: 50,
                    gas_limit: 0,
                    gas_price: 0,
                    data: vec![],
                },
                signature_type: SignatureType::Ed25519,
                signature: vec![0; 64],
                public_key: vec![0; 32],
            };

            let mut batch = StorageBatch::new();
            let res = state.apply_transaction(&tx, &mut batch);
            assert!(matches!(res, Err(StateError::NonceMismatch { .. })));
        }
        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn test_apply_transaction_insufficient_balance() {
        let path = temp_db_path();
        {
            let storage = Storage::open(&path).unwrap();
            let state = StateManager::new(storage);

            let sender_addr = Address::from_pubkey_hash([0x11; 20]);
            let recipient_addr = Address::from_pubkey_hash([0x22; 20]);

            let sender_acc = Account::new(100, Nonce(0));
            state.put_account(&sender_addr, &sender_acc).unwrap();

            let tx = TransactionEnvelope {
                payload: TransactionPayload {
                    nonce: Nonce(1),
                    sender: sender_addr,
                    recipient: recipient_addr,
                    amount: 200, // Exceeds balance
                    fee: 10,
                    gas_limit: 0,
                    gas_price: 0,
                    data: vec![],
                },
                signature_type: SignatureType::Ed25519,
                signature: vec![0; 64],
                public_key: vec![0; 32],
            };

            let mut batch = StorageBatch::new();
            let res = state.apply_transaction(&tx, &mut batch);
            assert!(matches!(res, Err(StateError::InsufficientBalance { .. })));
        }
        let _ = std::fs::remove_dir_all(&path);
    }
}
