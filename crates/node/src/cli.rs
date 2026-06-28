//! Command Line Interface (CLI) subcommand parser for the ARUNA node.

use std::net::SocketAddr;
use std::path::Path;
use aruna_primitives::TransactionEnvelope;

#[derive(Debug, Clone)]
pub enum CliCommand {
    Daemon {
        p2p_port: u16,
        rpc_port: u16,
        peer_addr: Option<SocketAddr>,
        block_time_secs: u64,
    },
    Submit {
        file_path: String,
    },
    Status,
    Block {
        height: u64,
    },
    Blocks,
    Help,
}

pub struct CliConfig {
    pub command: CliCommand,
}

impl CliConfig {
    pub fn parse() -> Result<Self, String> {
        let args: Vec<String> = std::env::args().collect();
        
        if args.len() > 1 && !args[1].starts_with("-") {
            let subcommand = &args[1];
            match subcommand.as_str() {
                "submit" => {
                    if args.len() < 3 {
                        return Err("Missing transaction JSON file. Usage: aruna-node submit <tx_json_file>".to_string());
                    }
                    Ok(Self {
                        command: CliCommand::Submit { file_path: args[2].clone() }
                    })
                }
                "status" => Ok(Self { command: CliCommand::Status }),
                "block" => {
                    if args.len() < 3 {
                        return Err("Missing block height. Usage: aruna-node block <height>".to_string());
                    }
                    let height = args[2].parse::<u64>()
                        .map_err(|_| format!("Invalid block height '{}'. Must be a non-negative integer.", args[2]))?;
                    Ok(Self { command: CliCommand::Block { height } })
                }
                "blocks" => Ok(Self { command: CliCommand::Blocks }),
                "help" | "-h" | "--help" => Ok(Self { command: CliCommand::Help }),
                other => Err(format!("Unknown subcommand '{}'. Run with 'help' for usage.", other)),
            }
        } else {
            // Parse daemon options
            let mut p2p_port = 9000;
            let mut rpc_port = 8080;
            let mut peer_addr = None;
            let mut block_time_secs = 30;

            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--p2p-port" => {
                        if i + 1 < args.len() {
                            p2p_port = args[i + 1].parse::<u16>()
                                .map_err(|_| format!("Invalid P2P port: {}", args[i+1]))?;
                            i += 2;
                        } else {
                            return Err("Missing value for --p2p-port".to_string());
                        }
                    }
                    "--rpc-port" => {
                        if i + 1 < args.len() {
                            rpc_port = args[i + 1].parse::<u16>()
                                .map_err(|_| format!("Invalid RPC port: {}", args[i+1]))?;
                            i += 2;
                        } else {
                            return Err("Missing value for --rpc-port".to_string());
                        }
                    }
                    "--peer" => {
                        if i + 1 < args.len() {
                            let addr: SocketAddr = args[i + 1].parse()
                                .map_err(|_| format!("Invalid peer address '{}'; must be IP:PORT format.", args[i+1]))?;
                            peer_addr = Some(addr);
                            i += 2;
                        } else {
                            return Err("Missing value for --peer".to_string());
                        }
                    }
                    "--block-time" => {
                        if i + 1 < args.len() {
                            block_time_secs = args[i + 1].parse::<u64>()
                                .map_err(|_| format!("Invalid block time: {}", args[i+1]))?;
                            i += 2;
                        } else {
                            return Err("Missing value for --block-time".to_string());
                        }
                    }
                    "--help" | "-h" | "help" => {
                        return Ok(Self { command: CliCommand::Help });
                    }
                    _ => {
                        i += 1;
                    }
                }
            }

            Ok(Self {
                command: CliCommand::Daemon { p2p_port, rpc_port, peer_addr, block_time_secs }
            })
        }
    }

    pub fn print_help() {
        println!("ARUNA Chain Inspection & Transaction CLI");
        println!("Usage:");
        println!("  aruna-node status        Display current chain height and tip block hash");
        println!("  aruna-node block <h>     Display the block hash at height <h>");
        println!("  aruna-node blocks        List all block heights from 1 to the current tip");
        println!("  aruna-node submit <file> Submit a signed transaction JSON file to the mempool");
        println!();
        println!("Daemon Options (when starting the node):");
        println!("  --p2p-port <port>        P2P listening port (default: 9000)");
        println!("  --rpc-port <port>        HTTP RPC listening port (default: 8080)");
        println!("  --peer <ip:port>         Bootstrap peer address to connect to");
        println!("  --block-time <secs>      Block production loop interval in seconds (default: 30)");
        println!("  aruna-node               Start the full node daemon (default)");
    }
}

pub fn handle_submit(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tx_json = std::fs::read_to_string(file_path)?;

    // Local validation to ensure the JSON matches the schema
    if let Err(e) = serde_json::from_str::<TransactionEnvelope>(&tx_json) {
        return Err(format!("Transaction file is not a valid JSON TransactionEnvelope: {:?}", e).into());
    }

    match submit_transaction_cli(&tx_json) {
        Ok(response_body) => {
            println!("{}", response_body.trim());
            Ok(())
        }
        Err(e) => {
            Err(format!("Could not connect to node RPC at 127.0.0.1:8080. Details: {:?}", e).into())
        }
    }
}

fn submit_transaction_cli(tx_json: &str) -> Result<String, Box<dyn std::error::Error>> {
    use std::io::{Write, Read};
    use std::net::TcpStream;

    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    let request = format!(
        "POST /tx HTTP/1.1\r\n\
         Host: 127.0.0.1:8080\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n\
         {}",
        tx_json.len(),
        tx_json
    );
    stream.write_all(request.as_bytes())?;
    
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    
    if let Some(body_start) = response.find("\r\n\r\n") {
        Ok(response[body_start + 4..].to_string())
    } else {
        Ok(response)
    }
}

pub fn handle_inspect_command(command: CliCommand) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = Path::new("./data_sumatera");
    let storage = match aruna_storage::Storage::open_read_only(db_path) {
        Ok(s) => s,
        Err(e) => {
            return Err(format!(
                "Error opening database in read-only mode: {:?}\n\
                 Please ensure the node has been started at least once to initialize the database.",
                e
            ).into());
        }
    };

    match command {
        CliCommand::Status => {
            let height = storage.get_chain_height()?.unwrap_or(0);
            let tip_hash = storage.get_best_block()?
                .map(|h| h.to_string())
                .unwrap_or_else(|| "none".to_string());
            
            println!("{{\"height\":{},\"tip\":\"{}\"}}", height, tip_hash);
        }
        CliCommand::Block { height } => {
            match storage.get_block_hash_by_height(height)? {
                Some(hash) => {
                    println!("{{\"height\":{},\"hash\":\"{}\"}}", height, hash);
                }
                None => {
                    return Err(format!("Block not found at height {}", height).into());
                }
            }
        }
        CliCommand::Blocks => {
            let height = storage.get_chain_height()?.unwrap_or(0);
            for h in 1..=height {
                println!("{}", h);
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
