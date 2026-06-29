//! ARUNA core node runner.
//! Loads genesis configuration from toml file, initializes RocksDB storage, and verifies ledger state.

use aruna_node::cli::{self, CliConfig, CliCommand};
use aruna_node::bootstrap;
use aruna_node::runtime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured tracing logging (RUST_LOG=info by default)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = match CliConfig::parse() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    match args.command {
        CliCommand::Help => {
            CliConfig::print_help();
            Ok(())
        }
        CliCommand::Submit { file_path } => {
            cli::handle_submit(&file_path)
        }
        CliCommand::Status | CliCommand::Block { .. } | CliCommand::Blocks => {
            cli::handle_inspect_command(args.command)
        }
        CliCommand::Daemon { p2p_port, rpc_port, peer_addr, block_time_secs } => {
            println!("ARUNA Core Node starting...");

            let db_dir = if p2p_port == 9000 {
                "./data_sumatera".to_string()
            } else {
                format!("./data_sumatera_{}", p2p_port)
            };
            let db_path = std::path::PathBuf::from(db_dir);

            let genesis_config = bootstrap::load_genesis_config()?;
            let storage = bootstrap::initialize_database(p2p_port, &genesis_config)?;

            let context = runtime::NodeContext::new(
                storage,
                p2p_port,
                rpc_port,
                genesis_config.genesis.chain_id,
                db_path,
                block_time_secs,
            );

            let runtime = runtime::NodeRuntime::new(context);
            runtime.run(peer_addr).await
        }
    }
}
