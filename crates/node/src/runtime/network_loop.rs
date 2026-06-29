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

    // 3. Connect to bootstrap peer if provided, otherwise fallback to Seed Node if database is empty
    if let Some(peer) = bootstrap_peer {
        context.p2p_manager.clone().connect_to_peer(peer);
    } else {
        let has_peers = context.p2p_manager.load_peers_from_file()
            .map(|list| !list.is_empty())
            .unwrap_or(false);

        if !has_peers && context.p2p_port != 9000 && context.p2p_manager.chain_id != 7777 {
            let seed_node: SocketAddr = "127.0.0.1:9000".parse().unwrap();
            println!("No active peers or bootstrap address provided. Connecting to Sumatera Seed Node: {}", seed_node);
            context.p2p_manager.clone().connect_to_peer(seed_node);
        }
    }
}
