//! ARUNA Network Indexer — configuration loader.
//!
//! Reads from `config/indexer.toml` (or a path passed via `--config`).
//!
//! # Example config/indexer.toml
//! ```toml
//! [indexer]
//! node_rpc_url     = "http://127.0.0.1:8080"
//! database_url     = "postgres://aruna:aruna@localhost:5432/aruna_explorer"
//! poll_interval_secs = 5
//! ```

use serde::Deserialize;

/// Top-level configuration file structure.
#[derive(Debug, Deserialize)]
pub struct RootConfig {
    pub indexer: IndexerConfig,
}

/// Indexer runtime configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct IndexerConfig {
    /// Base URL of the ARUNA Node RPC (e.g. `http://127.0.0.1:8080`).
    pub node_rpc_url: String,
    /// PostgreSQL connection string for the explorer database.
    pub database_url: String,
    /// How often (in seconds) to poll the node RPC for new blocks.
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
}

fn default_poll_interval() -> u64 { 5 }

impl IndexerConfig {
    /// Load config from the given TOML file path.
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let raw = std::fs::read_to_string(path)?;
        let root: RootConfig = toml::from_str(&raw)?;
        Ok(root.indexer)
    }

    /// Resolve config path: `--config <path>` arg, or default locations.
    pub fn resolve_path() -> String {
        let args: Vec<String> = std::env::args().collect();
        if let Some(pos) = args.iter().position(|a| a == "--config") {
            if let Some(path) = args.get(pos + 1) {
                return path.clone();
            }
        }
        // Default search order
        for candidate in &[
            "config/indexer.toml",
            "../config/indexer.toml",
            "../../config/indexer.toml",
        ] {
            if std::path::Path::new(candidate).exists() {
                return candidate.to_string();
            }
        }
        "config/indexer.toml".to_string()
    }
}
