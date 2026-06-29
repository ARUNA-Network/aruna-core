//! PostgreSQL database layer for the ARUNA Network Indexer.
//!
//! Uses runtime sqlx::query (non-macro) so the crate builds without
//! a live DATABASE_URL at compile time.
//!
//! Manages the connection pool and provides typed write methods used by the poller.
//! Migrations are embedded via `sqlx::migrate!` and run automatically on startup.

use sqlx::{PgPool, postgres::PgPoolOptions, FromRow};
use tracing::info;

/// Initialize the PostgreSQL connection pool and run pending migrations.
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Connecting to PostgreSQL: {}", database_url);
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrations complete.");

    Ok(pool)
}

#[derive(FromRow)]
struct MaxHeightRow { max: Option<i64> }

/// Returns the highest block height already indexed, or None if no blocks exist.
pub async fn get_indexed_height(pool: &PgPool) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query_as::<_, MaxHeightRow>("SELECT MAX(height) AS max FROM blocks")
        .fetch_optional(pool)
        .await?;
    Ok(row.and_then(|r| r.max))
}

/// Upsert a block row into the `blocks` table.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_block(
    pool: &PgPool,
    height: i64,
    hash: &str,
    prev_hash: &str,
    merkle_root: &str,
    state_root: &str,
    timestamp: i64,
    difficulty: i64,
    nonce: i64,
    version: i32,
    tx_count: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO blocks
            (height, hash, prev_hash, merkle_root, state_root, timestamp,
             difficulty, nonce, version, tx_count, indexed_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
        ON CONFLICT (height) DO UPDATE SET
            hash        = EXCLUDED.hash,
            prev_hash   = EXCLUDED.prev_hash,
            merkle_root = EXCLUDED.merkle_root,
            state_root  = EXCLUDED.state_root,
            timestamp   = EXCLUDED.timestamp,
            difficulty  = EXCLUDED.difficulty,
            nonce       = EXCLUDED.nonce,
            version     = EXCLUDED.version,
            tx_count    = EXCLUDED.tx_count,
            indexed_at  = NOW()
        "#
    )
    .bind(height).bind(hash).bind(prev_hash).bind(merkle_root).bind(state_root)
    .bind(timestamp).bind(difficulty).bind(nonce).bind(version).bind(tx_count)
    .execute(pool)
    .await?;
    Ok(())
}

/// Insert a transaction row. Ignores conflicts (idempotent).
#[allow(clippy::too_many_arguments)]
pub async fn insert_transaction(
    pool: &PgPool,
    hash: &str,
    block_height: i64,
    block_hash: &str,
    tx_index: i32,
    sender: &str,
    recipient: &str,
    amount: i64,
    fee: i64,
    nonce_val: i64,
    gas_limit: i64,
    gas_price: i64,
    has_data: bool,
    sig_type: i16,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO transactions
            (hash, block_height, block_hash, tx_index, sender, recipient,
             amount, fee, nonce_val, gas_limit, gas_price, has_data, sig_type, indexed_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW())
        ON CONFLICT (hash) DO NOTHING
        "#
    )
    .bind(hash).bind(block_height).bind(block_hash).bind(tx_index)
    .bind(sender).bind(recipient).bind(amount).bind(fee)
    .bind(nonce_val).bind(gas_limit).bind(gas_price).bind(has_data).bind(sig_type)
    .execute(pool)
    .await?;
    Ok(())
}

/// Upsert account balance/nonce. Only updates if `updated_at` (block height) is newer.
pub async fn upsert_account(
    pool: &PgPool,
    address: &str,
    balance: i64,
    nonce: i64,
    block_height: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO accounts (address, balance, nonce, updated_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (address) DO UPDATE SET
            balance    = EXCLUDED.balance,
            nonce      = EXCLUDED.nonce,
            updated_at = EXCLUDED.updated_at
        WHERE accounts.updated_at <= EXCLUDED.updated_at
        "#
    )
    .bind(address).bind(balance).bind(nonce).bind(block_height)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update the singleton chain_stats row.
pub async fn update_chain_stats(
    pool: &PgPool,
    height: i64,
    best_hash: &str,
    last_block_time: i64,
    tx_delta: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE chain_stats
        SET height          = $1,
            best_hash       = $2,
            last_block_time = $3,
            total_tx_count  = total_tx_count + $4,
            updated_at      = NOW()
        WHERE id = 1
        "#
    )
    .bind(height).bind(best_hash).bind(last_block_time).bind(tx_delta)
    .execute(pool)
    .await?;
    Ok(())
}
