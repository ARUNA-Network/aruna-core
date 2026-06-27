//! Modular Dependency Injection Context and runtime loops.

use std::sync::Arc;
use aruna_storage::Storage;
use aruna_state::StateManager;
use aruna_consensus::ConsensusEngine;
use aruna_mempool::Mempool;
use aruna_networking::P2PManager;

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
}

impl NodeContext {
    pub fn new(
        storage: Storage,
        p2p_port: u16,
        rpc_port: u16,
        chain_id: u32,
    ) -> Self {
        let state_manager = StateManager::new(storage.clone());
        let consensus_engine = ConsensusEngine::new(state_manager.clone(), storage.clone());
        let mempool = Arc::new(Mempool::new(50000));
        let p2p_manager = Arc::new(P2PManager::new(
            storage.clone(),
            consensus_engine.clone(),
            mempool.clone(),
            p2p_port,
            chain_id,
        ));

        Self {
            storage,
            state_manager,
            consensus_engine,
            mempool,
            p2p_manager,
            p2p_port,
            rpc_port,
        }
    }
}
