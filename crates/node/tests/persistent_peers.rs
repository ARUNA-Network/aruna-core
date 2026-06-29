use std::net::SocketAddr;
use std::sync::Arc;
use std::path::PathBuf;
use std::time::SystemTime;
use aruna_primitives::Address;
use aruna_storage::Storage;
use aruna_state::StateManager;
use aruna_consensus::ConsensusEngine;
use aruna_mempool::Mempool;
use aruna_networking::P2PManager;

struct TempDirCleaner {
    path: PathBuf,
}

impl Drop for TempDirCleaner {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn temp_db_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("aruna_persistent_peers_{}", nanos));
    path
}

#[test]
fn test_persistent_peer_management_load_save() {
    // 1. Setup temporary directory for database and peers.json
    let temp_path = temp_db_path();
    let _cleaner = TempDirCleaner { path: temp_path.clone() };
    
    let db_path = temp_path.join("db");
    let peers_file = temp_path.join("peers.json");

    let storage = Storage::open(&db_path).unwrap();
    let state_manager = StateManager::new(storage.clone());
    
    let miner_addr = Address::from_pubkey_hash([0x01; 20]);
    let validator_addr = Address::from_pubkey_hash([0x02; 20]);
    let treasury_addr = Address::from_pubkey_hash([0x03; 20]);
    let consensus_engine = ConsensusEngine::new(
        state_manager,
        storage.clone(),
        miner_addr,
        validator_addr,
        treasury_addr,
    );
    let mempool = Arc::new(Mempool::new(100));
    let node_id = [0x00; 32];

    // 2. Initialize P2PManager and check no peers are saved
    let p2p_manager = P2PManager::new(
        storage.clone(),
        consensus_engine.clone(),
        mempool.clone(),
        9000,
        1,
        node_id,
        Some(peers_file.clone()),
    );

    let loaded_peers = p2p_manager.load_peers_from_file().unwrap();
    assert!(loaded_peers.is_empty(), "Peers list must be empty initially");

    // 3. Save a peer address and assert file exists and is populated
    let peer_addr: SocketAddr = "127.0.0.1:9001".parse().unwrap();
    p2p_manager.save_peer(peer_addr);

    assert!(peers_file.exists(), "peers.json file must be created on save_peer");

    // 4. Simulate restart: Create a new P2PManager pointing to the same peers_file
    let p2p_manager_restarted = P2PManager::new(
        storage,
        consensus_engine,
        mempool,
        9000,
        1,
        node_id,
        Some(peers_file),
    );

    let recovered_peers = p2p_manager_restarted.load_peers_from_file().unwrap();
    assert_eq!(recovered_peers.len(), 1, "Must recover exactly 1 peer address");
    assert_eq!(recovered_peers[0], peer_addr, "Recovered peer address must match saved address");
}
