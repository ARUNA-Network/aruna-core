use std::sync::Arc;
use crate::rpc::AppState;
use super::NodeContext;

pub async fn start_rpc_server(context: Arc<NodeContext>) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState {
        storage: context.storage.clone(),
        mempool: context.mempool.clone(),
        p2p_manager: context.p2p_manager.clone(),
        consensus_engine: context.consensus_engine.clone(),
        db_path: context.db_path.clone(),
        start_time: std::time::Instant::now(),
        block_time_secs: context.block_time_secs,
        rpc_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
    };

    let app = crate::rpc::build_router(app_state);
    let rpc_addr = format!("127.0.0.1:{}", context.rpc_port);
    println!("Starting RPC server on {}...", rpc_addr);
    let listener = tokio::net::TcpListener::bind(&rpc_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
