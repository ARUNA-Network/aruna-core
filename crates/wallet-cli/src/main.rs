//! ARUNA Network Command-Line Wallet
//!
//! Provides key generation, address derivation, balance queries, transaction signing,
//! and transaction broadcasting against a running ARUNA node RPC endpoint.
//!
//! # Security
//! Private keys are never stored to disk. They must be provided as CLI flags for each
//! signing operation. All transaction signing occurs locally before broadcast.
//! NEVER pipe a private key from an untrusted source.

use std::io::{Read, Write};
use std::net::TcpStream;

use aruna_crypto::{Ed25519Keypair, derive_pubkey_hash};
use aruna_primitives::{
    Address, Nonce, SignatureType, TransactionEnvelope, TransactionPayload, serialize,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let result = match args[1].as_str() {
        "new"     => cmd_new(),
        "address" => cmd_address(&args),
        "balance" => cmd_balance(&args),
        "send"    => cmd_send(&args),
        "tx"      => cmd_tx(&args),
        "help" | "--help" | "-h" => { print_usage(); Ok(()) }
        other => Err(format!("Unknown subcommand: '{}'. Run 'aruna-wallet help'.", other)),
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn print_usage() {
    eprintln!(r#"ARUNA Network Wallet CLI

USAGE:
    aruna-wallet <COMMAND>

COMMANDS:
    new
        Generate a new Ed25519 keypair. Prints address (Bech32m) + public/private key hex.
        WARNING: Private key material is displayed once. Store it securely.

    address <pubkey-hex>
        Derive the Bech32m (sum1...) address from a 32-byte public key hex string.

    balance <address> --rpc <host:port>
        Query the balance and nonce of an address from a running node.

    send --privkey <hex> --to <address> --amount <u64> --fee <u64> --nonce <u64> --rpc <host:port>
        Sign and broadcast a transfer transaction. Prints the transaction hash on success.

    tx <tx-hash> --rpc <host:port>
        Query the status of a submitted transaction.

EXAMPLES:
    aruna-wallet new
    aruna-wallet address 034fa2ab...
    aruna-wallet balance sum1abc... --rpc 127.0.0.1:8080
    aruna-wallet send --privkey 7f3a... --to sum1xyz... --amount 1000 --fee 5000 --nonce 1 --rpc 127.0.0.1:8080
    aruna-wallet tx a1b2c3d4... --rpc 127.0.0.1:8080
"#);
}

// ── Subcommand implementations ────────────────────────────────────────────────

/// Generate a fresh Ed25519 keypair and print the address + key material.
fn cmd_new() -> Result<(), String> {
    let keypair = Ed25519Keypair::generate();
    let pubkey_bytes = keypair.public_key_bytes();
    let seed_bytes = keypair.seed_bytes();

    let pkh = derive_pubkey_hash(&pubkey_bytes);
    let address = Address::from_pubkey_hash(pkh);
    let addr_str = address.to_bech32m("sum")
        .map_err(|e| format!("Failed to encode address: {}", e))?;

    println!("=== New ARUNA Wallet ===");
    println!("Address (Bech32m):  {}", addr_str);
    println!("Public Key (hex):   {}", hex::encode(pubkey_bytes));
    println!();
    println!("WARNING — PRIVATE KEY — store securely, NEVER share:");
    println!("Seed (hex):         {}", hex::encode(seed_bytes));
    println!();
    println!("NOTE: Pass the seed hex as --privkey when signing transactions.");
    println!("ARUNA never stores private keys on disk.");
    Ok(())
}

/// Derive a Bech32m address from a 32-byte public key hex string.
fn cmd_address(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        return Err("Usage: aruna-wallet address <pubkey-hex>".to_string());
    }
    let pubkey_hex = &args[2];
    let pubkey_bytes: [u8; 32] = hex::decode(pubkey_hex)
        .map_err(|e| format!("Invalid hex public key: {}", e))?
        .try_into()
        .map_err(|_| "Public key must be exactly 32 bytes".to_string())?;

    let pkh = derive_pubkey_hash(&pubkey_bytes);
    let address = Address::from_pubkey_hash(pkh);
    let addr_str = address.to_bech32m("sum")
        .map_err(|e| format!("Address encoding failed: {}", e))?;

    println!("{}", addr_str);
    Ok(())
}

/// Query an address's balance and nonce from a running node RPC.
fn cmd_balance(args: &[String]) -> Result<(), String> {
    let addr_str = parse_positional(args, 2, "address")?;
    let rpc = parse_flag(args, "--rpc")?;

    let path = format!("/address/{}", addr_str);
    let body = http_get(&rpc, &path)?;

    let parsed: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse RPC response: {}. Body: {}", e, body))?;

    println!("Address: {}", addr_str);
    if let Some(balance) = parsed.get("balance") {
        println!("Balance: {} micro-ARU", balance);
    }
    if let Some(nonce) = parsed.get("nonce") {
        println!("Nonce:   {}", nonce);
    }
    if parsed.get("balance").is_none() {
        println!("Raw response: {}", body);
    }
    Ok(())
}

/// Sign and broadcast a transaction to a running node RPC.
fn cmd_send(args: &[String]) -> Result<(), String> {
    let privkey_hex = parse_flag(args, "--privkey")?;
    let to_str      = parse_flag(args, "--to")?;
    let amount: u64 = parse_flag(args, "--amount")?
        .parse().map_err(|_| "--amount must be a positive integer".to_string())?;
    let fee: u64    = parse_flag(args, "--fee")?
        .parse().map_err(|_| "--fee must be a positive integer".to_string())?;
    let nonce: u64  = parse_flag(args, "--nonce")?
        .parse().map_err(|_| "--nonce must be a positive integer (current nonce + 1)".to_string())?;
    let rpc         = parse_flag(args, "--rpc")?;

    // Reconstruct keypair from seed bytes
    let seed_bytes: [u8; 32] = hex::decode(&privkey_hex)
        .map_err(|e| format!("Invalid --privkey hex: {}", e))?
        .try_into()
        .map_err(|_| "Private key (seed) must be exactly 32 bytes".to_string())?;
    let keypair = Ed25519Keypair::from_seed(&seed_bytes);
    let pubkey = keypair.public_key_bytes();
    let pkh = derive_pubkey_hash(&pubkey);
    let sender = Address::from_pubkey_hash(pkh);

    // Decode recipient Bech32m address
    let (_, recipient) = Address::from_bech32m(&to_str)
        .map_err(|e| format!("Invalid --to address: {}", e))?;

    // Build and sign transaction payload
    let payload = TransactionPayload {
        nonce: Nonce(nonce),
        sender,
        recipient,
        amount,
        fee,
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };
    let payload_bytes = serialize(&payload)
        .map_err(|e| format!("Failed to serialize payload: {}", e))?;
    let signature = keypair.sign(&payload_bytes).to_vec();

    let tx = TransactionEnvelope {
        payload,
        signature_type: SignatureType::Ed25519,
        signature,
        public_key: pubkey.to_vec(),
    };

    // Compute tx hash from serialized envelope (matches node indexing)
    let tx_bytes = serialize(&tx)
        .map_err(|e| format!("Failed to serialize transaction: {}", e))?;
    let tx_hash = aruna_crypto::blake3_hash(&tx_bytes);

    // JSON-encode and POST to /tx
    let tx_json = serde_json::to_string(&tx)
        .map_err(|e| format!("Failed to JSON-encode transaction: {}", e))?;
    let response = http_post(&rpc, "/tx", &tx_json)?;

    println!("Transaction submitted.");
    println!("TX Hash: {}", hex::encode(tx_hash.0));
    println!("Node response: {}", response);
    Ok(())
}

/// Query a transaction by hash.
fn cmd_tx(args: &[String]) -> Result<(), String> {
    let hash_str = parse_positional(args, 2, "tx-hash")?;
    let rpc = parse_flag(args, "--rpc")?;

    let path = format!("/transaction/{}", hash_str);
    let body = http_get(&rpc, &path)?;

    let parsed: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse RPC response: {}. Body: {}", e, body))?;

    println!("TX Hash: {}", hash_str);
    if let Some(status) = parsed.get("status") {
        println!("Status:  {}", status);
    }
    if let Some(height) = parsed.get("block_height") {
        println!("Block:   {}", height);
    }
    if parsed.get("status").is_none() {
        println!("Raw response: {}", body);
    }
    Ok(())
}

// ── HTTP helpers ──────────────────────────────────────────────────────────────

fn http_get(host_port: &str, path: &str) -> Result<String, String> {
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, host_port
    );
    let mut stream = TcpStream::connect(host_port)
        .map_err(|e| format!("Cannot connect to '{}': {}", host_port, e))?;
    stream.write_all(request.as_bytes())
        .map_err(|e| format!("Failed to send GET: {}", e))?;
    let mut response = String::new();
    stream.read_to_string(&mut response)
        .map_err(|e| format!("Failed to read response: {}", e))?;
    extract_body(&response)
}

fn http_post(host_port: &str, path: &str, body: &str) -> Result<String, String> {
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, host_port, body.len(), body
    );
    let mut stream = TcpStream::connect(host_port)
        .map_err(|e| format!("Cannot connect to '{}': {}", host_port, e))?;
    stream.write_all(request.as_bytes())
        .map_err(|e| format!("Failed to send POST: {}", e))?;
    let mut response = String::new();
    stream.read_to_string(&mut response)
        .map_err(|e| format!("Failed to read response: {}", e))?;
    extract_body(&response)
}

fn extract_body(response: &str) -> Result<String, String> {
    Ok(response.find("\r\n\r\n")
        .map(|i| response[i + 4..].to_string())
        .unwrap_or_else(|| response.to_string()))
}

// ── CLI helpers ───────────────────────────────────────────────────────────────

fn parse_flag(args: &[String], flag: &str) -> Result<String, String> {
    for i in 0..args.len() {
        if args[i] == flag {
            return args.get(i + 1).cloned()
                .ok_or_else(|| format!("Flag '{}' requires a value", flag));
        }
    }
    Err(format!("Required flag '{}' is missing", flag))
}

fn parse_positional(args: &[String], pos: usize, name: &str) -> Result<String, String> {
    args.get(pos).cloned()
        .ok_or_else(|| format!("Missing required argument: <{}>", name))
}
