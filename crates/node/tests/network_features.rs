use std::net::SocketAddr;
use std::sync::Arc;
use std::path::PathBuf;
use std::time::SystemTime;
use aruna_primitives::{
    Address, Hash, BlockHeader, BlockBody, Difficulty,
    HeaderSyncRequestMessage
};
use aruna_storage::Storage;
use aruna_state::StateManager;
use aruna_consensus::ConsensusEngine;
use aruna_mempool::Mempool;
use aruna_networking::{P2PManager, P2PMessage};

struct TempDirCleaner {
    path: PathBuf,
}

impl Drop for TempDirCleaner {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn temp_db_path(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("aruna_network_features_{}_{}", suffix, nanos));
    path
}

fn build_test_context(port: u16, path: &std::path::Path) -> (Storage, ConsensusEngine, Arc<Mempool>, Arc<P2PManager>) {
    let storage = Storage::open(path).unwrap();
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
    let peers_file = path.join("peers.json");

    let p2p_manager = Arc::new(P2PManager::new(
        storage.clone(),
        consensus_engine.clone(),
        mempool.clone(),
        port,
        1,
        [0x00; 32],
        Some(peers_file),
    ));

    (storage, consensus_engine, mempool, p2p_manager)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_peer_discovery_mesh_propagation() {
    let path_a = temp_db_path("node_a");
    let path_b = temp_db_path("node_b");
    let _clean_a = TempDirCleaner { path: path_a.clone() };
    let _clean_b = TempDirCleaner { path: path_b.clone() };

    let (_, _, _, p2p_a) = build_test_context(19100, &path_a);
    let (_, _, _, p2p_b) = build_test_context(19101, &path_b);

    // Save a mock peer address (Node C) in Node A's registry
    let peer_c: SocketAddr = "127.0.0.1:19102".parse().unwrap();
    p2p_a.save_peer(peer_c);

    // Node A starts server
    p2p_a.clone().start_server();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Node B connects to Node A (which triggers handshake + GetPeersRequest automatically)
    let addr_a: SocketAddr = "127.0.0.1:19100".parse().unwrap();
    p2p_b.clone().connect_to_peer(addr_a);

    // Wait for handshake, request propagation and peer database serialization
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Assert that Node B successfully discovered Node C and stored it!
    let discovered_peers = p2p_b.load_peers_from_file().unwrap();
    assert!(discovered_peers.contains(&peer_c), "Node B must automatically discover Node C via Node A");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_header_sync_retrieval() {
    let path_a = temp_db_path("node_a_header");
    let path_b = temp_db_path("node_b_header");
    let _clean_a = TempDirCleaner { path: path_a.clone() };
    let _clean_b = TempDirCleaner { path: path_b.clone() };

    let (storage_a, _, _, p2p_a) = build_test_context(19200, &path_a);
    let (_, _, _, p2p_b) = build_test_context(19201, &path_b);

    // Commit genesis and a mock block header in Node A
    let genesis_header = BlockHeader {
        version: 1,
        prev_block_hash: Hash::zero(),
        merkle_root: Hash::zero(),
        state_root: Hash::zero(),
        timestamp: 1625097600,
        difficulty: Difficulty(1),
        nonce: 0,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let genesis_hash = aruna_crypto::blake3_hash(&aruna_primitives::serialize(&genesis_header).unwrap());
    storage_a.put_block_header(&genesis_hash, &genesis_header).unwrap();
    storage_a.put_block_body(&genesis_hash, &BlockBody { transactions: vec![], validator_metadata: vec![], ecosystem_metadata: vec![] }).unwrap();
    storage_a.put_best_block(&genesis_hash).unwrap();
    storage_a.put_chain_height(0).unwrap();
    storage_a.put_block_height_map(0, &genesis_hash).unwrap();
    storage_a.put_block_height_by_hash(&genesis_hash, 0).unwrap();

    let block1_header = BlockHeader {
        version: 1,
        prev_block_hash: genesis_hash,
        merkle_root: Hash::zero(),
        state_root: Hash::zero(),
        timestamp: 1625097630,
        difficulty: Difficulty(1),
        nonce: 100,
        validator_root: Hash::zero(),
        treasury_root: Hash::zero(),
    };
    let block1_hash = aruna_crypto::blake3_hash(&aruna_primitives::serialize(&block1_header).unwrap());
    storage_a.put_block_header(&block1_hash, &block1_header).unwrap();
    storage_a.put_block_body(&block1_hash, &BlockBody { transactions: vec![], validator_metadata: vec![], ecosystem_metadata: vec![] }).unwrap();
    storage_a.put_best_block(&block1_hash).unwrap();
    storage_a.put_chain_height(1).unwrap();
    storage_a.put_block_height_map(1, &block1_hash).unwrap();
    storage_a.put_block_height_by_hash(&block1_hash, 1).unwrap();

    // Node A starts server
    p2p_a.clone().start_server();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Node B connects to Node A
    let addr_a: SocketAddr = "127.0.0.1:19200".parse().unwrap();
    p2p_b.clone().connect_to_peer(addr_a);
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // We construct raw TCP connections to A
    let mut client_stream = tokio::net::TcpStream::connect(addr_a).await.unwrap();
    
    // Handshake
    let handshake = P2PMessage::Handshake(aruna_primitives::HandshakeMessage {
        version: 1,
        node_id: [0x00; 32],
        chain_id: aruna_primitives::ChainId(1),
        current_height: 0,
        capabilities: 1,
        listener_port: 19201,
    });
    aruna_networking::write_msg(&mut client_stream, &handshake).await.unwrap();
    let _reply = aruna_networking::read_msg(&mut client_stream).await.unwrap();

    // Send HeaderSyncRequest
    let req = P2PMessage::HeaderSyncRequest(HeaderSyncRequestMessage {
        start_height: 0,
        end_height: 1,
        limit: 100,
    });
    aruna_networking::write_msg(&mut client_stream, &req).await.unwrap();

    // Read response(s) until we get HeaderSyncResponse
    let mut response = None;
    for _ in 0..5 {
        let msg = aruna_networking::read_msg(&mut client_stream).await.unwrap();
        if let P2PMessage::HeaderSyncResponse(res) = msg {
            response = Some(res);
            break;
        } else {
            println!("Test client received other message: {:?}", msg);
        }
    }

    let res = response.expect("Did not receive HeaderSyncResponse");
    assert_eq!(res.status, 0);
    assert_eq!(res.headers.len(), 2);
    assert_eq!(res.headers[0].timestamp, 1625097600);
    assert_eq!(res.headers[1].timestamp, 1625097630);
}
