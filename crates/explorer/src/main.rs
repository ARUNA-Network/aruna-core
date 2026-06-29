//! ARUNA Network Explorer REST API
//!
//! Serves indexed chain data from PostgreSQL via a versioned REST API.
//! Reads ONLY from PostgreSQL — never touches RocksDB or the Node RPC directly.
//!
//! # ADR Reference
//! ADR-0017: Explorer Architecture — https://github.com/ARUNA-Network/aruna-core/docs/adr/adr-0017-explorer-architecture.md
//!
//! # Endpoints
//! - GET /health
//! - GET /api/v1/stats
//! - GET /api/v1/blocks?limit=&offset=
//! - GET /api/v1/block/latest
//! - GET /api/v1/block/height/:n
//! - GET /api/v1/block/hash/:hash
//! - GET /api/v1/transaction/:hash
//! - GET /api/v1/address/:addr?limit=&offset=
//! - GET /api/v1/search?q=

mod config;
mod db;
mod handlers;
mod models;

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("ARUNA Network Explorer API v{}", env!("CARGO_PKG_VERSION"));
    info!("ADR-0017: Reads from PostgreSQL — no RocksDB access.");

    // Load config
    let config_path = config::ExplorerConfig::resolve_path();
    info!("Loading configuration from: {}", config_path);
    let cfg = config::ExplorerConfig::from_file(&config_path).unwrap_or_else(|e| {
        eprintln!("Fatal: failed to load explorer config: {}", e);
        std::process::exit(1);
    });

    // Connect to PostgreSQL
    info!("Connecting to PostgreSQL...");
    let pool = db::connect(&cfg.database_url).await.unwrap_or_else(|e| {
        eprintln!("Fatal: failed to connect to PostgreSQL: {}", e);
        std::process::exit(1);
    });
    info!("PostgreSQL connected.");

    // CORS — allow all origins (public explorer API behind Cloudflare)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // Versioned API
        .route("/api/v1/stats",               get(handlers::get_stats))
        .route("/api/v1/blocks",              get(handlers::get_blocks))
        .route("/api/v1/block/latest",        get(handlers::get_block_latest))
        .route("/api/v1/block/height/{n}",    get(handlers::get_block_by_height))
        .route("/api/v1/block/hash/{hash}",   get(handlers::get_block_by_hash))
        .route("/api/v1/transaction/{hash}",  get(handlers::get_transaction))
        .route("/api/v1/address/{addr}",      get(handlers::get_address))
        .route("/api/v1/search",              get(handlers::search))
        .layer(cors)
        .with_state(pool);

    info!("Explorer API listening on http://{}", cfg.listen);
    let listener = tokio::net::TcpListener::bind(&cfg.listen).await
        .unwrap_or_else(|e| {
            eprintln!("Fatal: failed to bind {}: {}", cfg.listen, e);
            std::process::exit(1);
        });

    axum::serve(listener, app).await.expect("Server error");
}
