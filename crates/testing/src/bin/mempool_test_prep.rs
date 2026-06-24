use aruna_primitives::{Address, Hash, Nonce, TransactionPayload, TransactionEnvelope, SignatureType};
use aruna_storage::Storage;
use aruna_crypto::{Ed25519Keypair, derive_pubkey_hash};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Preparing mempool integration test...");

    // 1. Open the database in read-write mode
    let db_path = Path::new("./data_sumatera");
    if !db_path.exists() {
        return Err("Database does not exist at ./data_sumatera. Run the first startup step first.".into());
    }
    let storage = Storage::open(db_path)?;

    // 2. Generate test keypair
    let keypair = Ed25519Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let pkh = derive_pubkey_hash(&pubkey);
    
    let sender = Address::from_pubkey_hash(pkh);
    let sender_bech32 = sender.to_bech32m("sum")?;
    println!("Generated test address: {}", sender_bech32);

    // 3. Inject funded account state into RocksDB (10 ARU = 10,000,000 micro-ARU)
    let balance = 10_000_000;
    storage.put_account(&sender, balance, 0, &Hash::zero(), &Hash::zero())?;
    println!("Injected funded account into database with balance 10,000,000 micro-ARU");

    // 4. Construct a valid signed transaction (nonce 1, sending 1 ARU to a dummy address)
    let recipient = Address::from_pubkey_hash([0x88; 20]);
    let payload = TransactionPayload {
        nonce: Nonce(1),
        sender,
        recipient,
        amount: 1_000_000, // 1 ARU
        fee: 5_000,        // 5000 micro-ARU fee
        gas_limit: 0,
        gas_price: 0,
        data: vec![],
    };

    let payload_bytes = aruna_primitives::serialize(&payload)?;
    let signature = keypair.sign(&payload_bytes);

    let envelope = TransactionEnvelope {
        payload,
        signature_type: SignatureType::Ed25519,
        signature: signature.to_vec(),
        public_key: pubkey.to_vec(),
    };

    // 5. Serialize to JSON and write to test_tx.json
    let tx_json = serde_json::to_string_pretty(&envelope)?;
    std::fs::write("test_tx.json", tx_json)?;
    println!("Signed test transaction successfully written to test_tx.json");

    Ok(())
}
