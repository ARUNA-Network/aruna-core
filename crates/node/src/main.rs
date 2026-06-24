//! ARUNA core node runner.
//! Loads genesis configuration from toml file, initializes RocksDB storage, and verifies ledger state.

use aruna_primitives::{Block, BlockBody, BlockHeader, Hash, Address, Difficulty};
use aruna_storage::{Storage, StorageBatch};
use aruna_state::StateManager;
use aruna_consensus::ConsensusEngine;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct GenesisConfig {
    genesis: GenesisParameters,
    allocations: HashMap<String, u64>,
}

#[derive(Debug, Deserialize)]
struct GenesisParameters {
    version: u32,
    timestamp: u64,
    difficulty: u32,
    chain_id: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ARUNA Core Node starting...");

    // 1. Load genesis configuration from TOML file
    let config_path = Path::new("config/genesis.sumatera.toml");
    if !config_path.exists() {
        return Err(format!("Genesis configuration file not found at: {:?}", config_path).into());
    }
    let config_str = std::fs::read_to_string(config_path)?;
    let config: GenesisConfig = toml::from_str(&config_str)?;

    // 2. Establish data storage directory (Sumatera Testnet baseline)
    let db_path = Path::new("./data_sumatera");
    
    // 3. Open RocksDB Storage
    let storage = Storage::open(db_path)?;

    // 4. Initialize StateManager & ConsensusEngine
    let state_manager = StateManager::new(storage.clone());
    let _consensus_engine = ConsensusEngine::new(state_manager.clone(), storage.clone());

    // 5. Check if Genesis Block (Height 0) is already loaded
    let best_block = storage.get_best_block()?;
    
    if best_block.is_none() {
        println!("Initializing new ledger state from genesis config...");
        let mut batch = StorageBatch::new();

        // Dynamically apply allocations from TOML
        let m_aru = 1_000_000_u64; // 1 ARU = 1,000,000 micro-ARU
        for (address_str, amount_aru) in &config.allocations {
            let (_, addr) = Address::from_bech32m(address_str)?;
            let amount_micro = amount_aru.checked_mul(m_aru).ok_or("Allocation calculation overflow")?;
            batch.put_account(&addr, amount_micro, 0, &Hash::zero(), &Hash::zero());
        }

        // Construct Genesis Block Header from TOML parameters
        let genesis_header = BlockHeader {
            version: config.genesis.version,
            prev_block_hash: Hash::zero(),
            merkle_root: Hash::zero(),
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
        
        storage.write_batch(batch)?;
        
        // Save best/finalized metadata and chain_id
        storage.put_best_block(&genesis_hash)?;
        storage.put_chain_height(0)?;
        storage.put_finalized_block(&genesis_hash)?;
        storage.put_chain_id(config.genesis.chain_id)?;
    } else {
        println!("Genesis already initialized. Loading existing ledger state...");
    }

    // 6. Print Node Successful initialization banner
    println!("\nARUNA Node Started");
    println!("Network : Sumatera Testnet");
    println!("Height  : 0");
    println!("Genesis : Loaded");
    println!("Storage : Opened");
    println!("State   : Initialized\n");

    Ok(())
}
