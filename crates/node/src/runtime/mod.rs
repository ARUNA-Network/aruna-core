//! Modular Dependency Injection Context and runtime loops.

use std::sync::Arc;
use aruna_storage::Storage;
use aruna_state::StateManager;
use aruna_consensus::ConsensusEngine;
use aruna_mempool::Mempool;
use aruna_networking::P2PManager;
use aruna_primitives::Address;

pub mod node_runtime;
pub mod block_loop;
pub mod rpc_loop;
pub mod network_loop;
pub mod shutdown;

pub use node_runtime::NodeRuntime;

/// Dependency Injection Context container for ARUNA node.
#[allow(dead_code)]
pub struct NodeContext {
    pub storage: Storage,
    pub state_manager: StateManager,
    pub consensus_engine: ConsensusEngine,
    pub mempool: Arc<Mempool>,
    pub p2p_manager: Arc<P2PManager>,
    pub p2p_port: u16,
    pub rpc_port: u16,
    pub db_path: std::path::PathBuf,
    pub block_time_secs: u64,
}

impl NodeContext {
    pub fn new(
        storage: Storage,
        p2p_port: u16,
        rpc_port: u16,
        chain_id: u32,
        db_path: std::path::PathBuf,
        block_time_secs: u64,
    ) -> Self {
        // Testnet placeholder reward addresses.
        // Production nodes must load these from config.toml.
        let miner_addr = Address::from_pubkey_hash([0x01; 20]);
        let validator_addr = Address::from_pubkey_hash([0x02; 20]);
        let treasury_addr = Address::from_pubkey_hash([0x03; 20]);

        let state_manager = StateManager::new(storage.clone());
        let consensus_engine = ConsensusEngine::new(
            state_manager.clone(),
            storage.clone(),
            miner_addr,
            validator_addr,
            treasury_addr,
        );
        let mempool = Arc::new(Mempool::new(50000));

        // Derive node_id from a deterministic testnet placeholder public key.
        // Production nodes derive this from their Ed25519 node keypair loaded from config.
        let node_id = aruna_crypto::blake3_hash(&[0x00u8; 32]).0;

        let p2p_manager = Arc::new(P2PManager::new(
            storage.clone(),
            consensus_engine.clone(),
            mempool.clone(),
            p2p_port,
            chain_id,
            node_id,
            Some(db_path.join("peers.json")),
        ));

        Self {
            storage,
            state_manager,
            consensus_engine,
            mempool,
            p2p_manager,
            p2p_port,
            rpc_port,
            db_path,
            block_time_secs,
        }
    }
}
