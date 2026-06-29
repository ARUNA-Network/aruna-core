//! Asynchronous P2P networking and block synchronization layer for the ARUNA Network.
//! Uses a lightweight length-prefixed TCP protocol to exchange handshake, synchronization, and block broadcast messages.

use aruna_primitives::{Block, HandshakeMessage, SyncRequestMessage, SyncResponseMessage, ChainId, TransactionEnvelope};
use aruna_storage::Storage;
use aruna_consensus::ConsensusEngine;
use aruna_mempool::Mempool;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// A network message wrapper for length-prefixed P2P transmission.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum P2PMessage {
    Handshake(HandshakeMessage),
    SyncRequest(SyncRequestMessage),
    SyncResponse(SyncResponseMessage),
    BlockBroadcast(Block),
    TransactionBroadcast(TransactionEnvelope),
}

/// Helper function to write a length-prefixed P2PMessage over an asynchronous writer.
pub async fn write_msg<W>(stream: &mut W, msg: &P2PMessage) -> Result<(), std::io::Error>
where
    W: tokio::io::AsyncWrite + Unpin,
{
    let bytes = bincode::serialize(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let len = bytes.len() as u32;
    
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&bytes).await?;
    Ok(())
}

/// Helper function to read a length-prefixed P2PMessage from an asynchronous reader.
pub async fn read_msg<R>(stream: &mut R) -> Result<P2PMessage, std::io::Error>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    
    // Safety check: reject packets larger than 4 MB to prevent OOM / Huge Packet DDoS
    if len > 4 * 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Packet length {} exceeds maximum limit of 4 MB", len),
        ));
    }
    
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    
    let msg = bincode::deserialize(&buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(msg)
}

/// Thread-safe manager coordinating peers, block gossip, and synchronization.
pub struct P2PManager {
    storage: Storage,
    consensus: ConsensusEngine,
    mempool: Arc<Mempool>,
    p2p_port: u16,
    chain_id: u32,
    /// This node's unique identity: BLAKE3(node_public_key). Derived externally from the node keypair.
    node_id: [u8; 32],
    peer_writers: Arc<Mutex<Vec<mpsc::UnboundedSender<P2PMessage>>>>,
    connection_handles: Arc<Mutex<Vec<tokio::task::AbortHandle>>>,
    peer_addresses: Arc<Mutex<std::collections::HashSet<SocketAddr>>>,
    /// Highest block height reported by connected peers during P2P handshake.
    pub max_peer_height: Arc<std::sync::atomic::AtomicU64>,
}

impl P2PManager {
    /// Create a new P2PManager instance.
    ///
    /// # Arguments
    /// * `node_id` — BLAKE3 hash of the node's Ed25519 public key. Must be unique per node.
    pub fn new(storage: Storage, consensus: ConsensusEngine, mempool: Arc<Mempool>, p2p_port: u16, chain_id: u32, node_id: [u8; 32]) -> Self {
        Self {
            storage,
            consensus,
            mempool,
            p2p_port,
            chain_id,
            node_id,
            peer_writers: Arc::new(Mutex::new(Vec::new())),
            connection_handles: Arc::new(Mutex::new(Vec::new())),
            peer_addresses: Arc::new(Mutex::new(std::collections::HashSet::new())),
            max_peer_height: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Forcefully disconnect all currently connected active peers.
    pub fn disconnect_all(&self) {
        let mut handles = self.connection_handles.lock().unwrap();
        for handle in handles.iter() {
            handle.abort();
        }
        handles.clear();

        let mut writers = self.peer_writers.lock().unwrap();
        writers.clear();
        let mut addrs = self.peer_addresses.lock().unwrap();
        addrs.clear();
        println!("Forced disconnect of all peers on port {}", self.p2p_port);
    }

    /// Broadcast a block to all active connected peers.
    pub fn broadcast_block(&self, block: &Block) {
        self.broadcast_block_exclude(block, None);
    }

    /// Broadcast a block to all active connected peers (optionally excluding one peer).
    pub fn broadcast_block_exclude(&self, block: &Block, exclude_tx: Option<&mpsc::UnboundedSender<P2PMessage>>) {
        let msg = P2PMessage::BlockBroadcast(block.clone());
        let writers = self.peer_writers.lock().unwrap();
        for writer in writers.iter() {
            if let Some(exc) = exclude_tx {
                if writer.same_channel(exc) {
                    continue;
                }
            }
            let _ = writer.send(msg.clone());
        }
    }

    /// Broadcast a transaction to all active connected peers (optionally excluding one peer).
    pub fn broadcast_transaction(&self, tx: &TransactionEnvelope, exclude_tx: Option<&mpsc::UnboundedSender<P2PMessage>>) {
        let msg = P2PMessage::TransactionBroadcast(tx.clone());
        let writers = self.peer_writers.lock().unwrap();
        for writer in writers.iter() {
            if let Some(exc) = exclude_tx {
                if writer.same_channel(exc) {
                    continue;
                }
            }
            let _ = writer.send(msg.clone());
        }
    }

    /// Return the count of currently connected active peers.
    pub fn peer_count(&self) -> usize {
        self.peer_writers.lock().unwrap().len()
    }

    /// Return the list of currently connected active peer addresses.
    pub fn connected_peers(&self) -> Vec<SocketAddr> {
        let addrs = self.peer_addresses.lock().unwrap();
        addrs.iter().cloned().collect()
    }

    /// Return the maximum peer height reported.
    pub fn max_peer_height(&self) -> u64 {
        self.max_peer_height.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Starts the P2P server to listen for incoming connections.
    pub fn start_server(self: Arc<Self>) {
        let manager = self.clone();
        tokio::spawn(async move {
            let addr = format!("0.0.0.0:{}", manager.p2p_port);
            let listener = match TcpListener::bind(&addr).await {
                Ok(l) => {
                    println!("P2P server successfully listening on P2P port {}", manager.p2p_port);
                    l
                }
                Err(e) => {
                    eprintln!("Error binding P2P server to port {}: {:?}", manager.p2p_port, e);
                    return;
                }
            };

            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        let m = manager.clone();
                        let handle = tokio::spawn(async move {
                            if let Err(e) = m.handle_connection(stream, peer_addr, false).await {
                                eprintln!("P2P connection error with peer {}: {:?}", peer_addr, e);
                            }
                        });
                        manager.connection_handles.lock().unwrap().push(handle.abort_handle());
                    }
                    Err(e) => eprintln!("P2P server accept error: {:?}", e),
                }
            }
        });
    }

    /// Actively connect to a bootstrap peer.
    pub fn connect_to_peer(self: Arc<Self>, peer_addr: SocketAddr) {
        let manager = self.clone();
        let m = manager.clone();
        tokio::spawn(async move {
            println!("Actively connecting to peer: {}", peer_addr);
            match TcpStream::connect(peer_addr).await {
                Ok(stream) => {
                    let m_conn = m.clone();
                    let handle = tokio::spawn(async move {
                        if let Err(e) = m_conn.handle_connection(stream, peer_addr, true).await {
                            eprintln!("P2P connection error with peer {}: {:?}", peer_addr, e);
                        }
                    });
                    m.connection_handles.lock().unwrap().push(handle.abort_handle());
                }
                Err(e) => eprintln!("Failed to connect to peer {}: {:?}", peer_addr, e),
            }
        });
    }

    /// Handles a single peer connection (both inbound and outbound).
    /// Orchestrates handshakes, sync requests, sync responses, and gossip broadcasts.
    async fn handle_connection(
        &self,
        stream: TcpStream,
        peer_addr: SocketAddr,
        _is_outbound: bool,
    ) -> Result<(), std::io::Error> {
        let (mut reader, mut writer) = stream.into_split();
        let (tx, mut rx) = mpsc::unbounded_channel::<P2PMessage>();

        // Spawn a background task to handle outbound writes to the peer
        let writer_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = write_msg(&mut writer, &msg).await {
                    eprintln!("Failed to write P2P message: {:?}", e);
                    break;
                }
            }
        });

        // 1. Perform P2P Handshake immediately
        let our_height = self.storage.get_chain_height().unwrap_or(Some(0)).unwrap_or(0);
        let our_handshake = HandshakeMessage {
            version: 1,
            node_id: self.node_id,
            chain_id: ChainId(self.chain_id),
            current_height: our_height,
            capabilities: 1, // FULL_NODE
        };

        // Send our handshake via the channel
        tx.send(P2PMessage::Handshake(our_handshake))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::WriteZero, e.to_string()))?;

        // Read peer handshake
        let msg = read_msg(&mut reader).await?;
        let peer_handshake = match msg {
            P2PMessage::Handshake(h) => h,
            other => {
                writer_task.abort();
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Expected Handshake message, got: {:?}", other),
                ));
            }
        };

        // Validate Chain ID alignment
        if peer_handshake.chain_id.0 != self.chain_id {
            writer_task.abort();
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("Chain ID mismatch: expected {}, got {}", self.chain_id, peer_handshake.chain_id.0),
            ));
        }

        println!(
            "P2P Handshake successful with peer {}. Peer height: {}. Our height: {}",
            peer_addr, peer_handshake.current_height, our_height
        );

        // Add peer to writer registry
        {
            let mut writers = self.peer_writers.lock().unwrap();
            writers.push(tx.clone());
            let mut addrs = self.peer_addresses.lock().unwrap();
            addrs.insert(peer_addr);
        }

        // 2. Synchronization check
        self.max_peer_height.fetch_max(peer_handshake.current_height, std::sync::atomic::Ordering::Relaxed);
        
        // If peer height is greater than ours, initiate a SyncRequest to catch up
        if peer_handshake.current_height > our_height {
            // Start sync from a few blocks before our current height to handle potential forks/reorgs
            let start = our_height.saturating_sub(5).max(1);
            let end = peer_handshake.current_height;
            println!("Initiating sync request for blocks {} to {} from peer {}", start, end, peer_addr);
            
            let sync_req = SyncRequestMessage {
                start_height: start,
                end_height: end,
                block_limit: 500,
            };
            tx.send(P2PMessage::SyncRequest(sync_req))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::WriteZero, e.to_string()))?;
        }

        let mut loop_err = None;

        // 3. Main P2P message processing loop
        loop {
            match read_msg(&mut reader).await {
                Ok(p2p_msg) => match p2p_msg {
                    P2PMessage::Handshake(_) => {
                        eprintln!("Unexpected duplicate handshake from peer {}", peer_addr);
                        loop_err = Some(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Duplicate P2P Handshake received after connection establishment",
                        ));
                        break;
                    }
                    P2PMessage::SyncRequest(req) => {
                        if req.end_height < req.start_height {
                            loop_err = Some(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Invalid sync request: end_height {} is less than start_height {}", req.end_height, req.start_height),
                            ));
                            break;
                        }
                        if req.end_height - req.start_height > 500 {
                            loop_err = Some(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Invalid sync request: range {} blocks exceeds limit of 500 blocks", req.end_height - req.start_height),
                            ));
                            break;
                        }
                        // Peer is requesting blocks. Retrieve them from storage and send response.
                        println!("Received block sync request from peer {} for blocks {} to {}", peer_addr, req.start_height, req.end_height);
                        let mut blocks = Vec::new();
                        let mut status = 0; // Success

                        for h in req.start_height..=req.end_height {
                            match self.storage.get_block_hash_by_height(h) {
                                Ok(Some(hash)) => {
                                    let header_opt = self.storage.get_block_header(&hash).unwrap_or(None);
                                    let body_opt = self.storage.get_block_body(&hash).unwrap_or(None);
                                    if let (Some(header), Some(body)) = (header_opt, body_opt) {
                                        blocks.push(Block { header, body });
                                    } else {
                                        status = 2; // Internal Error (data missing)
                                        break;
                                    }
                                }
                                Ok(None) => {
                                    status = 1; // Out of Range
                                    break;
                                }
                                Err(_) => {
                                    status = 2; // Internal Error
                                    break;
                                }
                            }
                        }

                        let response = SyncResponseMessage { status, blocks };
                        tx.send(P2PMessage::SyncResponse(response))
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::WriteZero, e.to_string()))?;
                    }
                    P2PMessage::SyncResponse(res) => {
                        // Received block sync response. Validate and commit blocks sequentially.
                        if res.status == 0 {
                            println!("Received {} blocks from peer {}. Processing sync...", res.blocks.len(), peer_addr);
                            for block in res.blocks {
                                // Validate block (consensus signature & state rules)
                                if let Err(e) = self.consensus.validate_block(&block) {
                                    eprintln!("Sync block validation failed for height {}: {:?}", block.header.timestamp, e);
                                    break;
                                }
                                // Commit block to database
                                match self.consensus.commit_block(&block) {
                                    Ok(hash) => {
                                        let h = self.storage.get_chain_height().unwrap_or(Some(0)).unwrap_or(0);
                                        println!("Synced and committed Block #{} | Hash: {}", h, hash);
                                    }
                                    Err(e) => {
                                        eprintln!("Sync block commit failed: {:?}", e);
                                        break;
                                    }
                                }
                            }
                            println!("Synchronization complete. Current height: {}", self.storage.get_chain_height().unwrap_or(Some(0)).unwrap_or(0));
                        } else {
                            eprintln!("Peer {} returned sync error status: {}", peer_addr, res.status);
                        }
                    }
                    P2PMessage::BlockBroadcast(block) => {
                        // Received block broadcast (gossip) from peer.
                        let block_bytes = aruna_primitives::serialize(&block.header).unwrap();
                        let block_hash = aruna_crypto::blake3_hash(&block_bytes);
                        if self.storage.get_block_header(&block_hash).unwrap_or(None).is_some() {
                            // Already have this block, ignore silently early to save CPU
                            continue;
                        }

                        if let Err(e) = self.consensus.validate_block(&block) {
                            eprintln!("Broadcasted block validation failed from peer {}: {:?}", peer_addr, e);
                        } else {
                            match self.consensus.commit_block(&block) {
                                Ok(hash) => {
                                    let h = self.storage.get_chain_height().unwrap_or(Some(0)).unwrap_or(0);
                                    println!("Synced and committed Block #{} | Hash: {}", h, hash);
                                    // Relay the block broadcast to other peers, excluding the sender channel
                                    self.broadcast_block_exclude(&block, Some(&tx));
                                }
                                Err(e) => {
                                    eprintln!("Broadcasted block commit failed: {:?}", e);
                                }
                            }
                        }
                    }
                    P2PMessage::TransactionBroadcast(tx_envelope) => {
                        // Received transaction broadcast from peer.
                        // Validate and attempt to insert into local mempool.
                        match self.mempool.add_transaction(tx_envelope.clone(), &self.storage) {
                            Ok(hash) => {
                                println!("Received and added P2P transaction to mempool | Hash: {}", hash);
                                // Gossip (relay) the transaction to other peers, excluding the peer that sent it to us.
                                self.broadcast_transaction(&tx_envelope, Some(&tx));
                            }
                            Err(e) => {
                                // Silently ignore duplicates or already committed transactions.
                                match e {
                                    aruna_mempool::MempoolError::DuplicateNonce { .. } |
                                    aruna_mempool::MempoolError::NonceTooLow { .. } => {
                                        // common, ignore
                                    }
                                    other => {
                                        println!("P2P transaction validation rejected from peer {}: {:?}", peer_addr, other);
                                    }
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        println!("Peer {} disconnected gracefully", peer_addr);
                    } else {
                        eprintln!("Error reading P2P message from peer {}: {:?}", peer_addr, e);
                    }
                    break;
                }
            }
        }

        // Clean up peer writer registration
        {
            let mut writers = self.peer_writers.lock().unwrap();
            writers.retain(|w| !w.same_channel(&tx));
            let mut addrs = self.peer_addresses.lock().unwrap();
            addrs.remove(&peer_addr);
        }

        writer_task.abort();
        if let Some(err) = loop_err {
            return Err(err);
        }
        Ok(())
    }
}
