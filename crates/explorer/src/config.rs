//! ARUNA Network Explorer REST API — configuration.
//!
//! Reads from `config/explorer.toml` or a path from `--config` CLI arg.
//!
//! # Example config/explorer.toml
//! ```toml
//! [server]
//! listen       = "127.0.0.1:3000"
//! database_url = "postgres://aruna:aruna@localhost:5432/aruna_explorer"
//! ```

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RootConfig {
    pub server: ExplorerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExplorerConfig {
    /// Socket address to bind the HTTP server to.
    #[serde(default = "default_listen")]
    pub listen: String,
    /// PostgreSQL connection string.
    pub database_url: String,
}

fn default_listen() -> String { "127.0.0.1:3000".to_string() }

impl ExplorerConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let raw = std::fs::read_to_string(path)?;
        let root: RootConfig = toml::from_str(&raw)?;
        Ok(root.server)
    }

    pub fn resolve_path() -> String {
        let args: Vec<String> = std::env::args().collect();
        if let Some(pos) = args.iter().position(|a| a == "--config") {
            if let Some(path) = args.get(pos + 1) {
                return path.clone();
            }
        }
        for candidate in &[
            "config/explorer.toml",
            "../config/explorer.toml",
            "../../config/explorer.toml",
        ] {
            if std::path::Path::new(candidate).exists() {
                return candidate.to_string();
            }
        }
        "config/explorer.toml".to_string()
    }
}
