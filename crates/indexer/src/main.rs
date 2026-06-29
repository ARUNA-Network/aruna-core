//! ARUNA Network Indexer — entry point.
//!
//! Loads configuration, initializes the PostgreSQL connection pool (running
//! pending migrations), then starts the continuous polling loop against the
//! Node RPC.
//!
//! # Usage
//! ```
//! aruna-indexer [--config <path>]
//! ```
//!
//! Default config path: `config/indexer.toml`

mod config;
mod db;
mod poller;
mod rpc_client;

use tracing::info;

#[tokio::main]
async fn main() {
    // Initialise structured logging (RUST_LOG=info by default)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("ARUNA Network Indexer v{}", env!("CARGO_PKG_VERSION"));
    info!("ADR-0017: Explorer Architecture — reading Node RPC, writing to PostgreSQL.");

    // Load config
    let config_path = config::IndexerConfig::resolve_path();
    info!("Loading configuration from: {}", config_path);
    let cfg = config::IndexerConfig::from_file(&config_path)
        .unwrap_or_else(|e| {
            eprintln!("Fatal: failed to load indexer config: {}", e);
            std::process::exit(1);
        });

    info!("Node RPC URL    : {}", cfg.node_rpc_url);
    info!("Poll interval   : {}s", cfg.poll_interval_secs);

    // Initialize PostgreSQL pool + run migrations
    let pool = db::init_pool(&cfg.database_url)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Fatal: failed to connect to PostgreSQL: {}", e);
            std::process::exit(1);
        });

    // Build RPC client
    let rpc = rpc_client::RpcClient::new(&cfg.node_rpc_url);

    // Run polling loop forever
    poller::run(pool, rpc, cfg.poll_interval_secs).await;
}
