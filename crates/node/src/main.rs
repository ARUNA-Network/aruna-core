//! ARUNA core node runner.
//! Loads genesis configuration, initializes RocksDB storage, and verifies ledger state.

use aruna_primitives::{Block, BlockBody, BlockHeader, Hash, Address, Nonce, Difficulty};
use aruna_storage::{Storage, StorageBatch};
use aruna_state::{StateManager, Account};
use aruna_consensus::ConsensusEngine;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ARUNA Core Node starting...");

    // 1. Establish data storage directory (Sumatera Testnet baseline)
    let db_path = Path::new("./data_sumatera");
    
    // 2. Open RocksDB Storage
    let storage = Storage::open(db_path)?;

    // 3. Initialize StateManager & ConsensusEngine
    let state_manager = StateManager::new(storage.clone());
    let _consensus_engine = ConsensusEngine::new(state_manager.clone(), storage.clone());

    // 4. Check if Genesis Block (Height 0) is already loaded
    let best_block = storage.get_best_block()?;
    
    if best_block.is_none() {
        // Construct and persist Genesis Block & Allocations defined in docs/protocol/genesis.md
        let mut batch = StorageBatch::new();

        // Decode HRP addresses from genesis specification
        let (v1_hrp, v1_addr) = Address::from_bech32m("sum1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqquggh0p")?;
        let (_, v2_addr) = Address::from_bech32m("sum1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqquucj847")?;
        let (_, faucet_addr) = Address::from_bech32m("sum1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqp2e49c")?;
        let (_, treasury_addr) = Address::from_bech32m("sumc1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsq9lh6")?;
        let (_, founder_addr) = Address::from_bech32m("sum1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqd6ksp9")?;

        // 1.5% Premine: 5M ARU each to Validator 1, Validator 2, and Faucet
        let m_aru = 1_000_000_u64;
        batch.put_account(&v1_addr, 5_000_000 * m_aru, 0, &Hash::zero(), &Hash::zero());
        batch.put_account(&v2_addr, 5_000_000 * m_aru, 0, &Hash::zero(), &Hash::zero());
        batch.put_account(&faucet_addr, 5_000_000 * m_aru, 0, &Hash::zero(), &Hash::zero());

        // 5% Treasury Governance Allocation: 50M ARU
        batch.put_account(&treasury_addr, 50_000_000 * m_aru, 0, &Hash::zero(), &Hash::zero());

        // 1.5% Founder locked allocation: 15M ARU
        batch.put_account(&founder_addr, 15_000_000 * m_aru, 0, &Hash::zero(), &Hash::zero());

        // Construct Genesis Block Header
        let genesis_header = BlockHeader {
            version: 1,
            prev_block_hash: Hash::zero(),
            merkle_root: Hash::zero(),
            timestamp: 1782252000, // June 23, 2026, 22:00:00 UTC (Placeholder)
            difficulty: Difficulty(0x1e0ffff0),
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
        
        // Save best/finalized metadata
        storage.put_best_block(&genesis_hash)?;
        storage.put_chain_height(0)?;
        storage.put_finalized_block(&genesis_hash)?;
    }

    // 5. Print Node Successful initialization banner
    println!("\nARUNA Node Started");
    println!("Network : Sumatera Testnet");
    println!("Height  : 0");
    println!("Genesis : Loaded");
    println!("Storage : Opened");
    println!("State   : Initialized\n");

    Ok(())
}
