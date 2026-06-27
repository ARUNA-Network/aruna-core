//! Performance Benchmark binary for the ARUNA Network blockchain.
//! Measures transaction processing speed (TPS), block propagation times, 
//! historical block synchronization latencies, and memory footprint.

use aruna_primitives::{Block, BlockBody, BlockHeader, Hash, Difficulty, Address, Nonce, TransactionPayload, TransactionEnvelope, SignatureType};
use aruna_storage::{Storage, StorageBatch};
use aruna_state::StateManager;
use aruna_consensus::ConsensusEngine;
use aruna_crypto::{Ed25519Keypair, derive_pubkey_hash};
use aruna_primitives::serialize;
use std::path::PathBuf;
use std::time::{SystemTime, Instant};
use serde_json::json;

struct TempDirCleaner {
    path: PathBuf,
}

impl Drop for TempDirCleaner {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn temp_db_path(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("aruna_benchmark_{}_{}", suffix, nanos));
    path
}

fn get_memory_usage_mb() -> f64 {
    if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
        let parts: Vec<&str> = statm.split_whitespace().collect();
        if parts.len() > 1 {
            if let Ok(pages) = parts[1].parse::<u64>() {
                // RSS page size is typically 4KB
                return (pages as f64 * 4096.0) / (1024.0 * 1024.0);
            }
        }
    }
    0.0
}

fn get_directory_size(path: &std::path::Path) -> u64 {
    let mut total_size = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total_size += meta.len();
                } else if meta.is_dir() {
                    total_size += get_directory_size(&entry.path());
                }
            }
        }
    }
    total_size
}

fn initialize_node_state(path: &std::path::Path, sender: &Address) -> (Storage, StateManager, ConsensusEngine, Hash) {
    let storage = Storage::open(path).expect("Failed to open storage");
    let state_manager = StateManager::new(storage.clone());
    let engine = ConsensusEngine::new(state_manager.clone(), storage.clone());

    // Pre-fund sender
    let mut init_batch = StorageBatch::new();
    init_batch.put_account(sender, 10_000_000_000, 0, &Hash::zero(), &Hash::zero());
    storage.write_batch(init_batch).unwrap();

    // Genesis Block
    let genesis_header = BlockHeader {
        version: 1,
        prev_block_hash: Hash::zero(),
        merkle_root: Hash::zero(),
        state_root: Hash::zero(),
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
    let genesis_bytes = serialize(&genesis_header).unwrap();
    let genesis_hash = aruna_crypto::blake3_hash(&genesis_bytes);

    storage.put_block_header(&genesis_hash, &genesis_header).unwrap();
    storage.put_block_body(&genesis_hash, &genesis_body).unwrap();
    storage.put_best_block(&genesis_hash).unwrap();
    storage.put_chain_height(0).unwrap();
    storage.put_cumulative_difficulty(&genesis_hash, 0).unwrap();
    storage.put_block_height_by_hash(&genesis_hash, 0).unwrap();

    (storage, state_manager, engine, genesis_hash)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Starting ARUNA Performance Benchmark ===");

    let path_a = temp_db_path("node_a");
    let path_b = temp_db_path("node_b");
    let _cleaner_a = TempDirCleaner { path: path_a.clone() };
    let _cleaner_b = TempDirCleaner { path: path_b.clone() };

    let keypair = Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let pkh = derive_pubkey_hash(&pubkey);
    let sender = Address::from_pubkey_hash(pkh);

    let (_storage_a, _state_a, engine_a, genesis_hash) = initialize_node_state(&path_a, &sender);
    let (_storage_b, _state_b, engine_b, _) = initialize_node_state(&path_b, &sender);

    // 1. Generate 1,000 transaction envelopes
    println!("Generating 1,000 signed transactions...");
    let gen_start = Instant::now();
    let mut transactions = Vec::with_capacity(1000);
    for i in 1..=1000 {
        let recipient = Address::from_pubkey_hash([i as u8; 20]);
        let payload = TransactionPayload {
            nonce: Nonce(i),
            sender,
            recipient,
            amount: 100,
            fee: 10,
            gas_limit: 0,
            gas_price: 0,
            data: vec![],
        };
        let sig = keypair.sign(&serialize(&payload).unwrap()).to_vec();
        let envelope = TransactionEnvelope {
            payload,
            signature_type: SignatureType::Ed25519,
            signature: sig,
            public_key: pubkey.to_vec(),
        };
        transactions.push(envelope);
    }
    let gen_duration = gen_start.elapsed();

    // 2. Measure TPS (sequential block packaging & execution)
    println!("Benchmarking Transaction Processing Speed (TPS) and block production time...");
    let block_count = 10;
    let tx_per_block = 100;
    let mut produced_blocks = Vec::new();
    let mut parent_hash = genesis_hash;
    let mut parent_state_root = Hash::zero();

    let mut total_production_duration = std::time::Duration::new(0, 0);
    let tps_start = Instant::now();
    for b_idx in 0..block_count {
        let block_txs = transactions[(b_idx * tx_per_block)..((b_idx + 1) * tx_per_block)].to_vec();
        
        let prod_start = Instant::now();
        let body = BlockBody {
            transactions: block_txs,
            validator_metadata: vec![],
            ecosystem_metadata: vec![],
        };
        let merkle_root = ConsensusEngine::calculate_merkle_root(&body.transactions).unwrap();
        let mut header = BlockHeader {
            version: 1,
            prev_block_hash: parent_hash,
            merkle_root,
            state_root: Hash::zero(),
            timestamp: 1782252000 + (b_idx as u64 * 30),
            difficulty: Difficulty(504381424),
            nonce: 0,
            validator_root: Hash::zero(),
            treasury_root: Hash::zero(),
        };

        // Compute state root using dry-run
        let temp_block = Block { header: header.clone(), body: body.clone() };
        let state_root = engine_a.calculate_state_root(parent_state_root, &temp_block).unwrap();
        header.state_root = state_root;

        let block = Block { header, body };
        total_production_duration += prod_start.elapsed();

        let block_hash = engine_a.commit_block(&block).unwrap();
        
        parent_hash = block_hash;
        parent_state_root = state_root;
        produced_blocks.push((block_hash, block));
    }
    let tps_duration = tps_start.elapsed();
    let tps = 1000.0 / tps_duration.as_secs_f64();
    let avg_block_prod_ms = (total_production_duration.as_secs_f64() * 1000.0) / block_count as f64;

    // 3. Measure Block Propagation / Processing latency (1 block validation + commit)
    println!("Benchmarking single block propagation latency...");
    let prop_block_txs = transactions[0..tx_per_block].to_vec();
    let prop_body = BlockBody {
        transactions: prop_block_txs,
        validator_metadata: vec![],
        ecosystem_metadata: vec![],
    };
    let prop_merkle = ConsensusEngine::calculate_merkle_root(&prop_body.transactions).unwrap();
    let mut prop_header = BlockHeader {
        version: 1,
        prev_block_hash: genesis_hash,
        merkle_root: prop_merkle,
        state_root: Hash::zero(),
        timestamp: 1782252030,
        difficulty: Difficulty(504381424),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    // Re-create matching block state root on Node B
    let prop_temp = Block { header: prop_header.clone(), body: prop_body.clone() };
    let prop_state_root = engine_b.calculate_state_root(Hash::zero(), &prop_temp).unwrap();
    prop_header.state_root = prop_state_root;
    let prop_block = Block { header: prop_header, body: prop_body };

    let prop_start = Instant::now();
    let _ = engine_b.commit_block(&prop_block).unwrap();
    let prop_duration = prop_start.elapsed();

    // 4. Measure Historical Blockchain Synchronization (Node C syncs all 10 blocks)
    println!("Benchmarking historical block synchronization...");
    let path_c = temp_db_path("node_c");
    let _cleaner_c = TempDirCleaner { path: path_c.clone() };
    let (storage_c, _, engine_c, _) = initialize_node_state(&path_c, &sender);

    let sync_start = Instant::now();
    for (_hash, block) in &produced_blocks {
        let _ = engine_c.commit_block(block).unwrap();
    }
    let sync_duration = sync_start.elapsed();

    // Assert heights are synchronized
    assert_eq!(storage_c.get_chain_height().unwrap().unwrap(), 10);

    // 5. Measure Resident Set Size memory consumption
    let memory_used_mb = get_memory_usage_mb();

    // 6. Measure Database size on disk (Node A after committing 10 blocks)
    let db_size_bytes = get_directory_size(&path_a);
    let db_size_kb = db_size_bytes as f64 / 1024.0;

    // Print JSON Performance Benchmark Report
    let report = json!({
        "metrics": {
            "transactions_generated": 1000,
            "generation_duration_ms": gen_duration.as_millis(),
            "blocks_processed": block_count,
            "transactions_processed": 1000,
            "tps_execution_only": tps,
            "tps_total_duration_ms": tps_duration.as_millis(),
            "avg_block_production_duration_ms": avg_block_prod_ms,
            "single_block_propagation_ms": prop_duration.as_secs_f64() * 1000.0,
            "historical_sync_10_blocks_ms": sync_duration.as_millis(),
            "memory_footprint_rss_mb": memory_used_mb,
            "database_size_on_disk_kb": db_size_kb,
        },
        "success": true
    });

    println!("\n=== PERFORMANCE REPORT ===");
    println!("{}", serde_json::to_string_pretty(&report)?);
    println!("==========================");

    Ok(())
}
