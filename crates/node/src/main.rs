//! ARUNA core node runner.
//! Loads genesis configuration from toml file, initializes RocksDB storage, and verifies ledger state.

mod cli;
mod rpc;
mod bootstrap;
mod runtime;

use cli::{CliConfig, CliCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        CliCommand::Daemon { p2p_port, rpc_port, peer_addr } => {
            println!("ARUNA Core Node starting...");

            let genesis_config = bootstrap::load_genesis_config()?;
            let storage = bootstrap::initialize_database(p2p_port, &genesis_config)?;

            let context = runtime::NodeContext::new(
                storage,
                p2p_port,
                rpc_port,
                genesis_config.genesis.chain_id,
            );

            let runtime = runtime::NodeRuntime::new(context);
            runtime.run(peer_addr).await
        }
    }
}
