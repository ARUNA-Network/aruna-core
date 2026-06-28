use std::sync::Arc;
use aruna_primitives::Hash;
use super::NodeContext;

pub fn start_block_producer(context: Arc<NodeContext>) {
    tokio::spawn(async move {
        println!("Starting Block Producer loop ({}-second interval)...", context.block_time_secs);
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(context.block_time_secs)).await;
            
            // Fetch pending transactions from mempool
            let txs = context.mempool.get_pending_transactions(100);
            
            let current_height = match context.storage.get_chain_height() {
                Ok(h) => h.unwrap_or(0),
                Err(e) => {
                    eprintln!("Error reading chain height: {:?}", e);
                    continue;
                }
            };
            println!("Current Height: {}", current_height);

            match context.consensus_engine.produce_block(txs) {
                Ok(block) => {
                    let tx_count = block.body.transactions.len();
                    match context.consensus_engine.commit_block(&block) {
                        Ok(hash) => {
                            // Evict committed transactions from the mempool
                            let committed_hashes: Vec<Hash> = block.body.transactions.iter().map(|tx| {
                                let bytes = aruna_primitives::serialize(tx).unwrap();
                                aruna_crypto::blake3_hash(&bytes)
                            }).collect();
                            context.mempool.remove_transactions(&committed_hashes);
                            
                            // Broadcast the block to P2P peers!
                            context.p2p_manager.broadcast_block(&block);
                            
                            let height = match context.storage.get_chain_height() {
                                Ok(h) => h.unwrap_or(0),
                                Err(e) => {
                                    eprintln!("Error reading chain height: {:?}", e);
                                    continue;
                                }
                            };
                            println!("New Height: {}", height);
                            println!(
                                "Block #{} produced with {} transactions | Height: {} | Height={} | Hash: {}",
                                height, tx_count, height, height, hash
                            );
                        }
                        Err(e) => eprintln!("Error committing block: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Error producing block: {:?}", e),
            }
        }
    });
}
