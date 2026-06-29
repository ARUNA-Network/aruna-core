//! RocksDB storage wrapper for the ARUNA Network core database.
//! Conforms to specifications defined in docs/protocol/state.md.

use aruna_primitives::{Address, Hash, BlockHeader, BlockBody, serialize, deserialize};
use thiserror::Error;
use std::path::Path;
use std::sync::Arc;

const PREFIX_ACCOUNT: u8 = b'a';
const PREFIX_CODE: u8 = b'c';
const PREFIX_STORAGE: u8 = b's';
const PREFIX_HEADER: u8 = b'h';
const PREFIX_BODY: u8 = b'd';
const PREFIX_TX_INDEX: u8 = b't';
const PREFIX_HEIGHT_MAP: u8 = b'b';
const PREFIX_META: u8 = b'm';
const PREFIX_HASH_TO_HEIGHT: u8 = b'n';
const PREFIX_CUMULATIVE_DIFFICULTY: u8 = b'u';

/// Database storage error types.
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("RocksDB error: {0}")]
    Database(#[from] rocksdb::Error),
    #[error("Format/Serialization error: {0}")]
    Format(String),
    #[error("Missing expected record")]
    NotFound,
}

/// Type-safe thread-safe wrapper around RocksDB database.
#[derive(Clone)]
pub struct Storage {
    db: Arc<rocksdb::DB>,
}

impl Storage {
    /// Open or create a RocksDB store at the specified filesystem path.
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        // Optimize RocksDB for low-memory environments like ARM/Raspberry Pi
        opts.increase_parallelism(2);
        opts.set_max_background_jobs(2);

        let db = rocksdb::DB::open(&opts, path)?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Open a RocksDB store in read-only mode, permitting concurrent reads while another process holds the write lock.
    pub fn open_read_only(path: &Path) -> Result<Self, StorageError> {
        let mut opts = rocksdb::Options::default();
        // Optimize RocksDB for low-memory environments like ARM/Raspberry Pi
        opts.increase_parallelism(2);
        opts.set_max_background_jobs(2);

        let db = rocksdb::DB::open_for_read_only(&opts, path, false)?;
        Ok(Self { db: Arc::new(db) })
    }

    // --- Key Generators ---

    fn account_key(address: &Address) -> [u8; 33] {
        let mut key = [0u8; 33];
        key[0] = PREFIX_ACCOUNT;
        key[1..33].copy_from_slice(&address.0);
        key
    }

    fn code_key(code_hash: &Hash) -> [u8; 33] {
        let mut key = [0u8; 33];
        key[0] = PREFIX_CODE;
        key[1..33].copy_from_slice(&code_hash.0);
        key
    }

    fn storage_key(address: &Address, storage_key: &Hash) -> [u8; 65] {
        let mut key = [0u8; 65];
        key[0] = PREFIX_STORAGE;
        key[1..33].copy_from_slice(&address.0);
        key[33..65].copy_from_slice(&storage_key.0);
        key
    }

    fn header_key(hash: &Hash) -> [u8; 33] {
        let mut key = [0u8; 33];
        key[0] = PREFIX_HEADER;
        key[1..33].copy_from_slice(&hash.0);
        key
    }

    fn body_key(hash: &Hash) -> [u8; 33] {
        let mut key = [0u8; 33];
        key[0] = PREFIX_BODY;
        key[1..33].copy_from_slice(&hash.0);
        key
    }

    fn tx_index_key(tx_hash: &Hash) -> [u8; 33] {
        let mut key = [0u8; 33];
        key[0] = PREFIX_TX_INDEX;
        key[1..33].copy_from_slice(&tx_hash.0);
        key
    }

    fn height_map_key(height: u64) -> [u8; 9] {
        let mut key = [0u8; 9];
        key[0] = PREFIX_HEIGHT_MAP;
        key[1..9].copy_from_slice(&height.to_be_bytes());
        key
    }

    fn hash_to_height_key(hash: &Hash) -> [u8; 33] {
        let mut key = [0u8; 33];
        key[0] = PREFIX_HASH_TO_HEIGHT;
        key[1..33].copy_from_slice(&hash.0);
        key
    }

    fn cumulative_difficulty_key(hash: &Hash) -> [u8; 33] {
        let mut key = [0u8; 33];
        key[0] = PREFIX_CUMULATIVE_DIFFICULTY;
        key[1..33].copy_from_slice(&hash.0);
        key
    }

    fn meta_key(field: &[u8]) -> Vec<u8> {
        let mut key = Vec::with_capacity(1 + field.len());
        key.push(PREFIX_META);
        key.extend_from_slice(field);
        key
    }

    // --- Value Serializers ---

    fn serialize_account(balance: u64, nonce: u64, code_hash: &Hash, storage_root: &Hash) -> [u8; 80] {
        let mut value = [0u8; 80];
        value[0..8].copy_from_slice(&balance.to_be_bytes());
        value[8..16].copy_from_slice(&nonce.to_be_bytes());
        value[16..48].copy_from_slice(&code_hash.0);
        value[48..80].copy_from_slice(&storage_root.0);
        value
    }

    fn deserialize_account(bytes: &[u8]) -> Result<(u64, u64, Hash, Hash), StorageError> {
        if bytes.len() != 80 {
            return Err(StorageError::Format("Invalid account record size".to_string()));
        }
        let mut balance_bytes = [0u8; 8];
        balance_bytes.copy_from_slice(&bytes[0..8]);
        let balance = u64::from_be_bytes(balance_bytes);

        let mut nonce_bytes = [0u8; 8];
        nonce_bytes.copy_from_slice(&bytes[8..16]);
        let nonce = u64::from_be_bytes(nonce_bytes);

        let mut code_bytes = [0u8; 32];
        code_bytes.copy_from_slice(&bytes[16..48]);
        let code_hash = Hash(code_bytes);

        let mut storage_bytes = [0u8; 32];
        storage_bytes.copy_from_slice(&bytes[48..80]);
        let storage_root = Hash(storage_bytes);

        Ok((balance, nonce, code_hash, storage_root))
    }

    fn serialize_tx_index(block_hash: &Hash, index: u32) -> [u8; 36] {
        let mut value = [0u8; 36];
        value[0..32].copy_from_slice(&block_hash.0);
        value[32..36].copy_from_slice(&index.to_be_bytes());
        value
    }

    fn deserialize_tx_index(bytes: &[u8]) -> Result<(Hash, u32), StorageError> {
        if bytes.len() != 36 {
            return Err(StorageError::Format("Invalid tx index record size".to_string()));
        }
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&bytes[0..32]);
        let block_hash = Hash(hash_bytes);

        let mut idx_bytes = [0u8; 4];
        idx_bytes.copy_from_slice(&bytes[32..36]);
        let index = u32::from_be_bytes(idx_bytes);

        Ok((block_hash, index))
    }

    // --- Public API Read/Write Operators ---

    /// Write account state to RocksDB.
    pub fn put_account(
        &self,
        address: &Address,
        balance: u64,
        nonce: u64,
        code_hash: &Hash,
        storage_root: &Hash,
    ) -> Result<(), StorageError> {
        let key = Self::account_key(address);
        let val = Self::serialize_account(balance, nonce, code_hash, storage_root);
        self.db.put(key, val)?;
        Ok(())
    }

    /// Read account state from RocksDB.
    pub fn get_account(&self, address: &Address) -> Result<Option<(u64, u64, Hash, Hash)>, StorageError> {
        let key = Self::account_key(address);
        match self.db.get(key)? {
            Some(bytes) => {
                let res = Self::deserialize_account(&bytes)?;
                Ok(Some(res))
            }
            None => Ok(None),
        }
    }

    /// Write smart contract raw bytecode to RocksDB.
    pub fn put_code(&self, code_hash: &Hash, bytecode: &[u8]) -> Result<(), StorageError> {
        let key = Self::code_key(code_hash);
        self.db.put(key, bytecode)?;
        Ok(())
    }

    /// Read contract raw bytecode from RocksDB.
    pub fn get_code(&self, code_hash: &Hash) -> Result<Option<Vec<u8>>, StorageError> {
        let key = Self::code_key(code_hash);
        Ok(self.db.get(key)?)
    }

    /// Write contract internal storage value (EVM word).
    pub fn put_contract_storage(&self, address: &Address, storage_key: &Hash, value: &[u8; 32]) -> Result<(), StorageError> {
        let key = Self::storage_key(address, storage_key);
        self.db.put(key, value)?;
        Ok(())
    }

    /// Read contract internal storage value (EVM word).
    pub fn get_contract_storage(&self, address: &Address, storage_key: &Hash) -> Result<Option<[u8; 32]>, StorageError> {
        let key = Self::storage_key(address, storage_key);
        match self.db.get(key)? {
            Some(bytes) => {
                if bytes.len() != 32 {
                    return Err(StorageError::Format("Invalid contract storage slot length".to_string()));
                }
                let mut ret = [0u8; 32];
                ret.copy_from_slice(&bytes);
                Ok(Some(ret))
            }
            None => Ok(None),
        }
    }

    /// Write block header to RocksDB.
    pub fn put_block_header(&self, hash: &Hash, header: &BlockHeader) -> Result<(), StorageError> {
        let key = Self::header_key(hash);
        let val = serialize(header).map_err(|e| StorageError::Format(e.to_string()))?;
        self.db.put(key, val)?;
        Ok(())
    }

    /// Read block header from RocksDB.
    pub fn get_block_header(&self, hash: &Hash) -> Result<Option<BlockHeader>, StorageError> {
        let key = Self::header_key(hash);
        match self.db.get(key)? {
            Some(bytes) => {
                let header = deserialize(&bytes).map_err(|e| StorageError::Format(e.to_string()))?;
                Ok(Some(header))
            }
            None => Ok(None),
        }
    }

    /// Write block body to RocksDB.
    pub fn put_block_body(&self, hash: &Hash, body: &BlockBody) -> Result<(), StorageError> {
        let key = Self::body_key(hash);
        let val = serialize(body).map_err(|e| StorageError::Format(e.to_string()))?;
        self.db.put(key, val)?;
        Ok(())
    }

    /// Read block body from RocksDB.
    pub fn get_block_body(&self, hash: &Hash) -> Result<Option<BlockBody>, StorageError> {
        let key = Self::body_key(hash);
        match self.db.get(key)? {
            Some(bytes) => {
                let body = deserialize(&bytes).map_err(|e| StorageError::Format(e.to_string()))?;
                Ok(Some(body))
            }
            None => Ok(None),
        }
    }

    /// Write transaction index (lookup for tx hashes).
    pub fn put_tx_index(&self, tx_hash: &Hash, block_hash: &Hash, tx_index: u32) -> Result<(), StorageError> {
        let key = Self::tx_index_key(tx_hash);
        let val = Self::serialize_tx_index(block_hash, tx_index);
        self.db.put(key, val)?;
        Ok(())
    }

    /// Read transaction index.
    pub fn get_tx_index(&self, tx_hash: &Hash) -> Result<Option<(Hash, u32)>, StorageError> {
        let key = Self::tx_index_key(tx_hash);
        match self.db.get(key)? {
            Some(bytes) => {
                let index_info = Self::deserialize_tx_index(&bytes)?;
                Ok(Some(index_info))
            }
            None => Ok(None),
        }
    }

    /// Map block height to its header hash.
    pub fn put_block_height_map(&self, height: u64, hash: &Hash) -> Result<(), StorageError> {
        let key = Self::height_map_key(height);
        self.db.put(key, hash.0)?;
        Ok(())
    }

    /// Map block hash to its height.
    pub fn put_block_height_by_hash(&self, hash: &Hash, height: u64) -> Result<(), StorageError> {
        let key = Self::hash_to_height_key(hash);
        self.db.put(key, height.to_be_bytes())?;
        Ok(())
    }

    /// Write block cumulative difficulty.
    pub fn put_cumulative_difficulty(&self, hash: &Hash, diff: u128) -> Result<(), StorageError> {
        let key = Self::cumulative_difficulty_key(hash);
        self.db.put(key, diff.to_be_bytes())?;
        Ok(())
    }

    /// Read block cumulative difficulty.
    pub fn get_cumulative_difficulty(&self, hash: &Hash) -> Result<Option<u128>, StorageError> {
        let key = Self::cumulative_difficulty_key(hash);
        match self.db.get(key)? {
            Some(bytes) => {
                if bytes.len() != 16 {
                    return Err(StorageError::Format("Invalid cumulative difficulty length".to_string()));
                }
                let mut diff_bytes = [0u8; 16];
                diff_bytes.copy_from_slice(&bytes);
                Ok(Some(u128::from_be_bytes(diff_bytes)))
            }
            None => Ok(None),
        }
    }

    /// Retrieve block height for a given hash.
    pub fn get_block_height_by_hash(&self, hash: &Hash) -> Result<Option<u64>, StorageError> {
        let key = Self::hash_to_height_key(hash);
        match self.db.get(key)? {
            Some(bytes) => {
                if bytes.len() != 8 {
                    return Err(StorageError::Format("Invalid height length".to_string()));
                }
                let mut height_bytes = [0u8; 8];
                height_bytes.copy_from_slice(&bytes);
                Ok(Some(u64::from_be_bytes(height_bytes)))
            }
            None => Ok(None),
        }
    }

    /// Retrieve block hash for a given height.
    pub fn get_block_hash_by_height(&self, height: u64) -> Result<Option<Hash>, StorageError> {
        let key = Self::height_map_key(height);
        match self.db.get(key)? {
            Some(bytes) => {
                if bytes.len() != 32 {
                    return Err(StorageError::Format("Invalid block hash length".to_string()));
                }
                let mut hash_bytes = [0u8; 32];
                hash_bytes.copy_from_slice(&bytes);
                Ok(Some(Hash(hash_bytes)))
            }
            None => Ok(None),
        }
    }

    /// Save the hash of the best block to storage.
    pub fn put_best_block(&self, hash: &Hash) -> Result<(), StorageError> {
        let key = Self::meta_key(b"best_block");
        self.db.put(key, hash.0)?;
        Ok(())
    }

    /// Read the hash of the best block from storage.
    pub fn get_best_block(&self) -> Result<Option<Hash>, StorageError> {
        let key = Self::meta_key(b"best_block");
        match self.db.get(key)? {
            Some(bytes) => {
                if bytes.len() != 32 {
                    return Err(StorageError::Format("Invalid hash length".to_string()));
                }
                let mut hash_bytes = [0u8; 32];
                hash_bytes.copy_from_slice(&bytes);
                Ok(Some(Hash(hash_bytes)))
            }
            None => Ok(None),
        }
    }

    /// Save the current chain height to storage.
    pub fn put_chain_height(&self, height: u64) -> Result<(), StorageError> {
        let key = Self::meta_key(b"chain_height");
        self.db.put(key, height.to_be_bytes())?;
        Ok(())
    }

    /// Read the current chain height from storage.
    pub fn get_chain_height(&self) -> Result<Option<u64>, StorageError> {
        let key = Self::meta_key(b"chain_height");
        match self.db.get(key)? {
            Some(bytes) => {
                if bytes.len() != 8 {
                    return Err(StorageError::Format("Invalid height length".to_string()));
                }
                let mut height_bytes = [0u8; 8];
                height_bytes.copy_from_slice(&bytes);
                Ok(Some(u64::from_be_bytes(height_bytes)))
            }
            None => Ok(None),
        }
    }

    /// Save the hash of the finalized block to storage.
    pub fn put_finalized_block(&self, hash: &Hash) -> Result<(), StorageError> {
        let key = Self::meta_key(b"finalized_block");
        self.db.put(key, hash.0)?;
        Ok(())
    }

    /// Read the hash of the finalized block from storage.
    pub fn get_finalized_block(&self) -> Result<Option<Hash>, StorageError> {
        let key = Self::meta_key(b"finalized_block");
        match self.db.get(key)? {
            Some(bytes) => {
                if bytes.len() != 32 {
                    return Err(StorageError::Format("Invalid hash length".to_string()));
                }
                let mut hash_bytes = [0u8; 32];
                hash_bytes.copy_from_slice(&bytes);
                Ok(Some(Hash(hash_bytes)))
            }
            None => Ok(None),
        }
    }

    /// Save the chain ID to storage.
    pub fn put_chain_id(&self, chain_id: u32) -> Result<(), StorageError> {
        let key = Self::meta_key(b"chain_id");
        self.db.put(key, chain_id.to_be_bytes())?;
        Ok(())
    }

    /// Read the chain ID from storage.
    pub fn get_chain_id(&self) -> Result<Option<u32>, StorageError> {
        let key = Self::meta_key(b"chain_id");
        match self.db.get(key)? {
            Some(bytes) => {
                if bytes.len() != 4 {
                    return Err(StorageError::Format("Invalid chain_id length".to_string()));
                }
                let mut id_bytes = [0u8; 4];
                id_bytes.copy_from_slice(&bytes);
                Ok(Some(u32::from_be_bytes(id_bytes)))
            }
            None => Ok(None),
        }
    }

    /// Execute atomic batch updates.
    pub fn write_batch(&self, batch: StorageBatch) -> Result<(), StorageError> {
        self.db.write(batch.batch)?;
        Ok(())
    }

    /// Flush all memtables of the RocksDB database to SST files on disk.
    pub fn flush(&self) -> Result<(), StorageError> {
        self.db.flush()?;
        Ok(())
    }

    /// Create a consistent point-in-time snapshot/checkpoint of the database at the specified path.
    pub fn create_checkpoint(&self, path: &Path) -> Result<(), StorageError> {
        let checkpoint = rocksdb::checkpoint::Checkpoint::new(&self.db)?;
        checkpoint.create_checkpoint(path)?;
        Ok(())
    }
}

/// Helper structure to compose atomic batch updates.
#[derive(Default)]
pub struct StorageBatch {
    batch: rocksdb::WriteBatch,
}

impl StorageBatch {
    /// Create a new empty batch.
    pub fn new() -> Self {
        Self { batch: rocksdb::WriteBatch::default() }
    }

    /// Add account state to batch.
    pub fn put_account(
        &mut self,
        address: &Address,
        balance: u64,
        nonce: u64,
        code_hash: &Hash,
        storage_root: &Hash,
    ) {
        let key = Storage::account_key(address);
        let val = Storage::serialize_account(balance, nonce, code_hash, storage_root);
        self.batch.put(key, val);
    }

    /// Add contract storage update to batch.
    pub fn put_contract_storage(&mut self, address: &Address, storage_key: &Hash, value: &[u8; 32]) {
        let key = Storage::storage_key(address, storage_key);
        self.batch.put(key, value);
    }

    /// Add transaction index to batch.
    pub fn put_tx_index(&mut self, tx_hash: &Hash, block_hash: &Hash, tx_index: u32) {
        let key = Storage::tx_index_key(tx_hash);
        let val = Storage::serialize_tx_index(block_hash, tx_index);
        self.batch.put(key, val);
    }

    /// Add block height index mapping to batch.
    pub fn put_block_height_map(&mut self, height: u64, hash: &Hash) {
        let key = Storage::height_map_key(height);
        self.batch.put(key, hash.0);
    }

    /// Add block hash to height mapping to batch.
    pub fn put_block_height_by_hash(&mut self, hash: &Hash, height: u64) {
        let key = Storage::hash_to_height_key(hash);
        self.batch.put(key, height.to_be_bytes());
    }

    /// Add cumulative difficulty to batch.
    pub fn put_cumulative_difficulty(&mut self, hash: &Hash, diff: u128) {
        let key = Storage::cumulative_difficulty_key(hash);
        self.batch.put(key, diff.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aruna_primitives::Difficulty;

    fn temp_db_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        // Generate a unique folder name
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("aruna_db_test_{}", time));
        path
    }

    #[test]
    fn test_storage_account_roundtrip() {
        let path = temp_db_path();
        {
            let storage = Storage::open(&path).unwrap();
            let address = Address::from_pubkey_hash([0x1a; 20]);
            let balance = 123456789;
            let nonce = 3;
            let code_hash = Hash([0x22; 32]);
            let storage_root = Hash([0x33; 32]);

            // Save and verify None
            assert!(storage.get_account(&address).unwrap().is_none());

            // Write account
            storage.put_account(&address, balance, nonce, &code_hash, &storage_root).unwrap();

            // Load and verify
            let (b, n, c, s) = storage.get_account(&address).unwrap().unwrap();
            assert_eq!(b, balance);
            assert_eq!(n, nonce);
            assert_eq!(c, code_hash);
            assert_eq!(s, storage_root);
        }
        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn test_storage_block_roundtrip() {
        let path = temp_db_path();
        {
            let storage = Storage::open(&path).unwrap();
            let block_hash = Hash([0xaa; 32]);
            let header = BlockHeader {
                version: 1,
                prev_block_hash: Hash([0xbb; 32]),
                merkle_root: Hash([0xcc; 32]),
                state_root: Hash([0xaa; 32]),
                timestamp: 123456789,
                difficulty: Difficulty(500),
                nonce: 99999,
                validator_root: Hash([0xdd; 32]),
                treasury_root: Hash([0xee; 32]),
            };

            storage.put_block_header(&block_hash, &header).unwrap();
            let loaded_header = storage.get_block_header(&block_hash).unwrap().unwrap();
            assert_eq!(loaded_header, header);
        }
        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn test_batch_write() {
        let path = temp_db_path();
        {
            let storage = Storage::open(&path).unwrap();
            let mut batch = StorageBatch::new();

            let addr1 = Address::from_pubkey_hash([0xaa; 20]);
            let addr2 = Address::from_pubkey_hash([0xbb; 20]);
            let code = Hash([0; 32]);
            let storage_root = Hash([0; 32]);

            batch.put_account(&addr1, 100, 1, &code, &storage_root);
            batch.put_account(&addr2, 200, 2, &code, &storage_root);

            storage.write_batch(batch).unwrap();

            let (b1, _, _, _) = storage.get_account(&addr1).unwrap().unwrap();
            let (b2, _, _, _) = storage.get_account(&addr2).unwrap().unwrap();
            assert_eq!(b1, 100);
            assert_eq!(b2, 200);
        }
        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn test_storage_hash_to_height_roundtrip() {
        let path = temp_db_path();
        {
            let storage = Storage::open(&path).unwrap();
            let block_hash = Hash([0xaa; 32]);
            let height = 123456;

            assert!(storage.get_block_height_by_hash(&block_hash).unwrap().is_none());
            storage.put_block_height_by_hash(&block_hash, height).unwrap();
            assert_eq!(storage.get_block_height_by_hash(&block_hash).unwrap().unwrap(), height);
        }
        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn test_cumulative_difficulty_roundtrip() {
        let path = temp_db_path();
        {
            let storage = Storage::open(&path).unwrap();
            let block_hash = Hash([0xbb; 32]);
            let diff = 98765432109876543210_u128;

            assert!(storage.get_cumulative_difficulty(&block_hash).unwrap().is_none());
            storage.put_cumulative_difficulty(&block_hash, diff).unwrap();
            assert_eq!(storage.get_cumulative_difficulty(&block_hash).unwrap().unwrap(), diff);
        }
        let _ = std::fs::remove_dir_all(&path);
    }
}
