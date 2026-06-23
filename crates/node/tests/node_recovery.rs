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
