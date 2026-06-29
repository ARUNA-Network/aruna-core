use std::sync::Arc;
use std::net::SocketAddr;
use super::NodeContext;

pub async fn start_p2p(context: Arc<NodeContext>, bootstrap_peer: Option<SocketAddr>) {
    // 1. Start P2P networking server
    context.p2p_manager.clone().start_server();

    // 2. Load and reconnect to persistent peers
    if let Ok(peers) = context.p2p_manager.load_peers_from_file() {
        if !peers.is_empty() {
            println!("Loaded {} persistent peers from peers.json. Reconnecting...", peers.len());
            for peer in peers {
                context.p2p_manager.clone().connect_to_peer(peer);
            }
        }
    }

    // 3. Connect to bootstrap peer if provided
    if let Some(peer) = bootstrap_peer {
        context.p2p_manager.clone().connect_to_peer(peer);
    }
}
