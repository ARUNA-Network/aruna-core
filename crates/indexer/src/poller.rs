//! Core polling loop for the ARUNA Network Indexer.
//!
//! Runs in an infinite async loop, comparing the locally indexed height
//! against the node's canonical height. For each new block:
//!   1. Fetches the full block from the Node RPC.
//!   2. Computes a deterministic transaction hash (BLAKE3 of sender+recipient+amount+nonce).
//!   3. Upserts the block, all transactions, and touched accounts into PostgreSQL.
//!   4. Updates the chain_stats singleton.

use sqlx::PgPool;
use tracing::{info, warn, error, debug};
use crate::rpc_client::{RpcClient, RpcTransaction};
use crate::db;

/// Run the indexer polling loop forever, until the process is killed.
pub async fn run(pool: PgPool, rpc: RpcClient, poll_interval_secs: u64) {
    let interval = std::time::Duration::from_secs(poll_interval_secs);
    info!("Indexer started. Polling every {} seconds.", poll_interval_secs);

    loop {
        if let Err(e) = poll_once(&pool, &rpc).await {
            error!("Polling error: {}", e);
        }
        tokio::time::sleep(interval).await;
    }
}

/// Execute one polling round.
async fn poll_once(pool: &PgPool, rpc: &RpcClient) -> Result<(), Box<dyn std::error::Error>> {
    // Fetch node canonical height
    let status = match rpc.get_status().await {
        Ok(s) => s,
        Err(e) => {
            warn!("Node RPC unreachable: {}", e);
            return Ok(());
        }
    };

    let node_height = status.height;

    // Determine the next height to index
    let indexed_height: u64 = match db::get_indexed_height(pool).await? {
        Some(h) => (h + 1) as u64,
        None => 0,
    };

    if indexed_height > node_height {
        debug!("Already at tip (indexed={}, node={}). Nothing to do.", indexed_height, node_height);
        return Ok(());
    }

    // Index up to 100 blocks per poll to avoid blocking for too long
    let end_height = (indexed_height + 99).min(node_height);
    info!(
        "Indexing blocks {}..={} (node tip={}).",
        indexed_height, end_height, node_height
    );

    for h in indexed_height..=end_height {
        index_block(pool, rpc, h).await?;
    }

    Ok(())
}

/// Fetch a single block from the Node RPC and write it to PostgreSQL.
async fn index_block(pool: &PgPool, rpc: &RpcClient, height: u64) -> Result<(), Box<dyn std::error::Error>> {
    let block = match rpc.get_block_by_height(height).await {
        Ok(b) => b,
        Err(e) => {
            warn!("Failed to fetch block at height {}: {}", height, e);
            return Ok(()); // skip, will retry next poll
        }
    };

    let hdr = &block.header;
    let tx_count = block.body.transactions.len() as i32;

    debug!("Indexing block #{} hash={}", height, block.hash);

    // Upsert block
    db::upsert_block(
        pool,
        height as i64,
        &block.hash,
        &hdr.prev_block_hash,
        &hdr.merkle_root,
        &hdr.state_root,
        hdr.timestamp as i64,
        hdr.difficulty_u64() as i64,
        hdr.nonce as i64,
        hdr.version as i32,
        tx_count,
    ).await?;

    // Index transactions
    for (idx, tx) in block.body.transactions.iter().enumerate() {
        let tx_hash = compute_tx_hash(tx, height, idx as u32);
        let sender    = tx.payload.sender_hex();
        let recipient = tx.payload.recipient_hex();
        let has_data  = !tx.payload.data.is_empty();
        let nonce_val = tx.payload.nonce_u64();
        let sig_type  = tx.sig_type_u8();

        db::insert_transaction(
            pool,
            &tx_hash,
            height as i64,
            &block.hash,
            idx as i32,
            &sender,
            &recipient,
            tx.payload.amount as i64,
            tx.payload.fee as i64,
            nonce_val as i64,
            tx.payload.gas_limit as i64,
            tx.payload.gas_price as i64,
            has_data,
            sig_type as i16,
        ).await?;

        // Upsert sender & recipient accounts
        for addr_str in [&sender, &recipient] {
            if addr_str.chars().all(|c| c == '0') || addr_str.is_empty() {
                continue;
            }
            match rpc.get_account(addr_str).await {
                Ok(acc) => {
                    db::upsert_account(
                        pool,
                        &acc.address,
                        acc.balance as i64,
                        acc.nonce as i64,
                        height as i64,
                    ).await?;
                }
                Err(e) => {
                    debug!("Could not fetch account {}: {}", addr_str, e);
                }
            }
        }
    }

    // Update chain_stats singleton
    db::update_chain_stats(
        pool,
        height as i64,
        &block.hash,
        hdr.timestamp as i64,
        tx_count as i64,
    ).await?;

    info!("Indexed block #{} ({} txs).", height, tx_count);
    Ok(())
}

/// Compute a deterministic transaction hash.
///
/// BLAKE3 over canonical encoding: `sender|recipient|amount|nonce|height|tx_index`
fn compute_tx_hash(tx: &RpcTransaction, block_height: u64, tx_index: u32) -> String {
    let canonical = format!(
        "{}|{}|{}|{}|{}|{}",
        tx.payload.sender_hex(),
        tx.payload.recipient_hex(),
        tx.payload.amount,
        tx.payload.nonce_u64(),
        block_height,
        tx_index,
    );
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}
