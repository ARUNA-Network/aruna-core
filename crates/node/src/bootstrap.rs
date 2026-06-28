//! Bootstrap and database initialization logic for the ARUNA node.

use std::collections::HashMap;
use std::path::Path;
use serde::Deserialize;
use aruna_primitives::{Address, Hash, Difficulty, Block, BlockHeader, BlockBody};
use aruna_storage::{Storage, StorageBatch};
use aruna_consensus::ConsensusEngine;

#[derive(Debug, Deserialize)]
pub struct GenesisConfig {
    pub genesis: GenesisParameters,
    pub allocations: HashMap<String, u64>,
}

#[derive(Debug, Deserialize)]
pub struct GenesisParameters {
    pub version: u32,
    pub timestamp: u64,
    pub difficulty: u32,
    pub chain_id: u32,
}

pub fn load_genesis_config() -> Result<GenesisConfig, Box<dyn std::error::Error>> {
    let config_path = Path::new("config/genesis.sumatera.toml");
    if !config_path.exists() {
        return Err(format!("Genesis configuration file not found at: {:?}", config_path).into());
    }
    let config_str = std::fs::read_to_string(config_path)?;
    let config: GenesisConfig = toml::from_str(&config_str)?;
    Ok(config)
}

pub fn initialize_database(
    p2p_port: u16,
    config: &GenesisConfig,
) -> Result<Storage, Box<dyn std::error::Error>> {
    // Establish data storage directory (dynamic depending on P2P port to allow local multi-node testing)
    let db_dir = if p2p_port == 9000 {
        "./data_sumatera".to_string()
    } else {
        format!("./data_sumatera_{}", p2p_port)
    };
    let db_path = Path::new(&db_dir);
    let storage = Storage::open(db_path)?;

    // Check if Genesis Block (Height 0) is already loaded
    let best_block = storage.get_best_block()?;
    
    if best_block.is_none() {
        println!("Initializing new ledger state from genesis config...");
        let mut batch = StorageBatch::new();

        // Dynamically apply allocations from TOML
        let m_aru = 1_000_000_u64; // 1 ARU = 1,000,000 micro-ARU
        let mut initial_accounts = std::collections::HashMap::new();

        for (address_str, amount_aru) in &config.allocations {
            let (_, addr) = Address::from_bech32m(address_str)?;
            let amount_micro = amount_aru.checked_mul(m_aru).ok_or("Allocation calculation overflow")?;
            batch.put_account(&addr, amount_micro, 0, &Hash::zero(), &Hash::zero());
            
            // Build the initial accounts set to calculate genesis state root
            let account = aruna_state::Account::new(amount_micro, aruna_primitives::Nonce::zero());
            initial_accounts.insert(addr, account);
        }

        // Calculate Genesis State Root Commitment (ADR-0016)
        let genesis_state_root = ConsensusEngine::compute_state_root_from_updates(Hash::zero(), &initial_accounts)?;

        // Construct Genesis Block Header from TOML parameters
        let genesis_header = BlockHeader {
            version: config.genesis.version,
            prev_block_hash: Hash::zero(),
            merkle_root: Hash::zero(),
            state_root: genesis_state_root,
            timestamp: config.genesis.timestamp,
            difficulty: Difficulty(config.genesis.difficulty),
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
        let header_bytes = aruna_primitives::serialize(&genesis_header)?;
        let genesis_hash = aruna_crypto::blake3_hash(&header_bytes);

        // Persist Block 0 data
        storage.put_block_header(&genesis_hash, &genesis_header)?;
        storage.put_block_body(&genesis_hash, &genesis_block.body)?;

        // Update Chain Metadata indexes in storage batch
        batch.put_block_height_map(0, &genesis_hash);
        batch.put_block_height_by_hash(&genesis_hash, 0);
        
        storage.write_batch(batch)?;
        
        // Save best/finalized metadata and chain_id
        storage.put_best_block(&genesis_hash)?;
        storage.put_chain_height(0)?;
        storage.put_finalized_block(&genesis_hash)?;
        storage.put_chain_id(config.genesis.chain_id)?;

        // Store genesis cumulative difficulty = 0.
        // This is mandatory: commit_block() for block 1 reads the parent's
        // cumulative difficulty to compute the chain's total accumulated work.
        // A missing entry here causes "Parent cumulative difficulty missing"
        // on every block production attempt.
        storage.put_cumulative_difficulty(&genesis_hash, 0)?;

    } else {
        println!("Genesis already initialized. Loading existing ledger state...");
    }

    // --- Self-Healing Index Backfill ---
    // Repair any indexes that may be missing from older DB snapshots.
    let best_height = storage.get_chain_height()?.unwrap_or(0);
    for h in 0..=best_height {
        if let Some(hash) = storage.get_block_hash_by_height(h)? {
            // Backfill height-by-hash index
            if storage.get_block_height_by_hash(&hash)?.is_none() {
                println!("Backfilling block hash→height index for block #{} ({})", h, hash);
                storage.put_block_height_by_hash(&hash, h)?;
            }
            // Backfill cumulative difficulty — recover DBs missing the genesis entry.
            // Only the genesis block (height 0) has a known baseline of 0.
            // For all other heights we cannot reconstruct without traversing headers.
            if h == 0 && storage.get_cumulative_difficulty(&hash)?.is_none() {
                println!("Backfilling missing cumulative difficulty for genesis block");
                storage.put_cumulative_difficulty(&hash, 0)?;
            }
        }
    }

    Ok(storage)
}
