//! Dependency Injection Context and Service Runtime orchestration for the ARUNA node.

use std::sync::Arc;
use std::net::SocketAddr;
use aruna_primitives::Hash;
use aruna_storage::Storage;
use aruna_state::StateManager;
use aruna_consensus::ConsensusEngine;
use aruna_mempool::Mempool;
use aruna_networking::P2PManager;
use crate::rpc::AppState;

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

pub struct NodeServices {
    context: Arc<NodeContext>,
}

impl NodeServices {
    pub fn new(context: Arc<NodeContext>) -> Self {
        Self { context }
    }

    /// Starts the block producer loop in a background thread.
    pub fn start_block_producer(&self) {
        let context = self.context.clone();
        
        tokio::spawn(async move {
            println!("Starting Block Producer loop (30-second interval)...");
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                
                // Fetch pending transactions from mempool
                let txs = context.mempool.get_pending_transactions(100);
                
                let current_height = match context.storage.get_chain_height() {
                    Ok(h) => h.unwrap_or(0),
                    Err(e) => {
                        eprintln!("Error reading chain height: {:?}", e);
                        continue;
                    }
                };
                println!("Current Height: {}", current_height);

                match context.consensus_engine.produce_block(txs) {
                    Ok(block) => {
                        let tx_count = block.body.transactions.len();
                        match context.consensus_engine.commit_block(&block) {
                            Ok(hash) => {
                                // Evict committed transactions from the mempool
                                let committed_hashes: Vec<Hash> = block.body.transactions.iter().map(|tx| {
                                    let bytes = aruna_primitives::serialize(tx).unwrap();
                                    aruna_crypto::blake3_hash(&bytes)
                                }).collect();
                                context.mempool.remove_transactions(&committed_hashes);
                                
                                // Broadcast the block to P2P peers!
                                context.p2p_manager.broadcast_block(&block);
                                
                                let height = match context.storage.get_chain_height() {
                                    Ok(h) => h.unwrap_or(0),
                                    Err(e) => {
                                        eprintln!("Error reading chain height: {:?}", e);
                                        continue;
                                    }
                                };
                                println!("New Height: {}", height);
                                println!(
                                    "Block #{} produced with {} transactions | Height: {} | Height={} | Hash: {}",
                                    height, tx_count, height, height, hash
                                );
                            }
                            Err(e) => eprintln!("Error committing block: {:?}", e),
                        }
                    }
                    Err(e) => eprintln!("Error producing block: {:?}", e),
                }
            }
        });
    }

    /// Starts the HTTP RPC Server.
    pub async fn start_rpc_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app_state = AppState {
            storage: self.context.storage.clone(),
            mempool: self.context.mempool.clone(),
            p2p_manager: self.context.p2p_manager.clone(),
        };

        let app = crate::rpc::build_router(app_state);
        let rpc_addr = format!("127.0.0.1:{}", self.context.rpc_port);
        println!("Starting RPC server on {}...", rpc_addr);
        let listener = tokio::net::TcpListener::bind(&rpc_addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

pub struct NodeRuntime {
    context: Arc<NodeContext>,
}

impl NodeRuntime {
    pub fn new(context: NodeContext) -> Self {
        Self {
            context: Arc::new(context),
        }
    }

    pub async fn run(&self, bootstrap_peer: Option<SocketAddr>) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nARUNA Node Started");
        println!("Network : Sumatera Testnet");
        println!("Height  : 0");
        println!("Genesis : Loaded");
        println!("Storage : Opened");
        println!("State   : Initialized\n");

        // 1. Start P2P networking server
        self.context.p2p_manager.clone().start_server();

        // 2. Connect to bootstrap peer if provided
        if let Some(peer) = bootstrap_peer {
            self.context.p2p_manager.clone().connect_to_peer(peer);
        }

        // 3. Initialize background services
        let services = NodeServices::new(self.context.clone());
        services.start_block_producer();

        // 4. Run HTTP RPC Server on main thread
        services.start_rpc_server().await?;

        Ok(())
    }
}
