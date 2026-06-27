use std::sync::Arc;
use std::net::SocketAddr;
use super::NodeContext;

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
        println!("Genesis : Loaded");
        println!("Storage : Opened");
        println!("State   : Initialized\n");

        // 1. Start P2P networking server
        super::network_loop::start_p2p(self.context.clone(), bootstrap_peer).await;

        // 2. Start block production loop
        super::block_loop::start_block_producer(self.context.clone());

        // 3. Prepare RPC task and Shutdown signals
        let rpc_context = self.context.clone();
        let rpc_task = tokio::spawn(async move {
            if let Err(e) = super::rpc_loop::start_rpc_server(rpc_context).await {
                eprintln!("RPC Server failed: {:?}", e);
            }
        });

        let shutdown_task = tokio::spawn(async {
            super::shutdown::listen_shutdown().await;
        });

        // 4. Select on RPC completion or Shutdown signal
        tokio::select! {
            _ = rpc_task => {
                println!("RPC service stopped.");
            }
            _ = shutdown_task => {
                println!("Graceful shutdown initiated. Exiting...");
            }
        }

        Ok(())
    }
}
