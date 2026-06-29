//! PostgreSQL read layer for the Explorer REST API.
//!
//! All functions use `sqlx::query_as` (runtime-verified) so the crate compiles
//! without a live PostgreSQL connection. No compile-time query macros.
//! No RocksDB access — reads exclusively from PostgreSQL.

use sqlx::{PgPool, FromRow};
use crate::models::*;

// ── Derive FromRow for model types ────────────────────────────────────────────
//
// sqlx::query_as::<_, T>() requires T: FromRow.
// We derive it for all models used in SELECT queries.

#[derive(FromRow)]
struct BlockRow {
    pub height: i64,
    pub hash: String,
    pub prev_hash: String,
    pub merkle_root: String,
    pub state_root: String,
    pub timestamp: i64,
    pub difficulty: i64,
    pub nonce: i64,
    pub version: i32,
    pub tx_count: i32,
}

impl From<BlockRow> for BlockSummary {
    fn from(r: BlockRow) -> Self {
        BlockSummary {
            height: r.height, hash: r.hash, prev_hash: r.prev_hash,
            merkle_root: r.merkle_root, state_root: r.state_root,
            timestamp: r.timestamp, difficulty: r.difficulty, nonce: r.nonce,
            version: r.version, tx_count: r.tx_count,
        }
    }
}

#[derive(FromRow)]
struct TxRow {
    pub hash: String,
    pub block_height: i64,
    pub block_hash: String,
    pub tx_index: i32,
    pub sender: String,
    pub recipient: String,
    pub amount: i64,
    pub fee: i64,
    pub nonce_val: i64,
    pub gas_limit: i64,
    pub gas_price: i64,
    pub has_data: bool,
    pub sig_type: i16,
}

impl From<TxRow> for TxSummary {
    fn from(r: TxRow) -> Self {
        TxSummary {
            hash: r.hash, block_height: r.block_height, block_hash: r.block_hash,
            tx_index: r.tx_index, sender: r.sender, recipient: r.recipient,
            amount: r.amount, fee: r.fee, nonce_val: r.nonce_val,
            gas_limit: r.gas_limit, gas_price: r.gas_price,
            has_data: r.has_data, sig_type: r.sig_type,
        }
    }
}

#[derive(FromRow)]
struct AccountRow {
    pub address: String,
    pub balance: i64,
    pub nonce: i64,
    pub updated_at: i64,
}

#[derive(FromRow)]
struct StatsRow {
    pub height: i64,
    pub total_tx_count: i64,
    pub best_hash: String,
    pub last_block_time: i64,
}

#[derive(FromRow)]
struct HashRow { pub hash: String }
#[derive(FromRow)]
struct HeightRow { pub height: i64 }
#[derive(FromRow)]
struct AddressRow { pub address: String }
#[derive(FromRow)]
struct CountRow { pub cnt: Option<i64> }
#[derive(FromRow)]
struct MaxRow { pub max: Option<i64> }

// ── Pool ─────────────────────────────────────────────────────────────────────

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
}

// ── Chain Stats ───────────────────────────────────────────────────────────────

pub async fn get_chain_stats(pool: &PgPool) -> Result<ChainStats, sqlx::Error> {
    let row = sqlx::query_as::<_, StatsRow>(
        "SELECT height, total_tx_count, best_hash, last_block_time FROM chain_stats WHERE id = 1"
    )
    .fetch_one(pool)
    .await?;

    Ok(ChainStats {
        height: row.height,
        total_tx_count: row.total_tx_count,
        best_hash: row.best_hash,
        last_block_time: row.last_block_time,
    })
}

// ── Blocks ────────────────────────────────────────────────────────────────────

pub async fn get_blocks(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<BlockSummary>, sqlx::Error> {
    let rows = sqlx::query_as::<_, BlockRow>(
        "SELECT height, hash, prev_hash, merkle_root, state_root, \
                timestamp, difficulty, nonce, version, tx_count \
         FROM blocks ORDER BY height DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit).bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(BlockSummary::from).collect())
}

pub async fn count_blocks(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) AS cnt FROM blocks")
        .fetch_one(pool).await?;
    Ok(row.cnt.unwrap_or(0))
}

pub async fn get_block_by_height(pool: &PgPool, height: i64) -> Result<Option<BlockDetail>, sqlx::Error> {
    let row = sqlx::query_as::<_, BlockRow>(
        "SELECT height, hash, prev_hash, merkle_root, state_root, \
                timestamp, difficulty, nonce, version, tx_count \
         FROM blocks WHERE height = $1"
    )
    .bind(height)
    .fetch_optional(pool)
    .await?;

    let Some(r) = row else { return Ok(None); };
    let txs = get_txs_for_block(pool, r.height).await?;
    Ok(Some(block_row_to_detail(r, txs)))
}

pub async fn get_block_by_hash(pool: &PgPool, hash: &str) -> Result<Option<BlockDetail>, sqlx::Error> {
    let row = sqlx::query_as::<_, BlockRow>(
        "SELECT height, hash, prev_hash, merkle_root, state_root, \
                timestamp, difficulty, nonce, version, tx_count \
         FROM blocks WHERE hash = $1"
    )
    .bind(hash)
    .fetch_optional(pool)
    .await?;

    let Some(r) = row else { return Ok(None); };
    let txs = get_txs_for_block(pool, r.height).await?;
    Ok(Some(block_row_to_detail(r, txs)))
}

pub async fn get_latest_block(pool: &PgPool) -> Result<Option<BlockDetail>, sqlx::Error> {
    let row = sqlx::query_as::<_, BlockRow>(
        "SELECT height, hash, prev_hash, merkle_root, state_root, \
                timestamp, difficulty, nonce, version, tx_count \
         FROM blocks ORDER BY height DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    let Some(r) = row else { return Ok(None); };
    let txs = get_txs_for_block(pool, r.height).await?;
    Ok(Some(block_row_to_detail(r, txs)))
}

fn block_row_to_detail(r: BlockRow, txs: Vec<TxSummary>) -> BlockDetail {
    BlockDetail {
        height: r.height, hash: r.hash, prev_hash: r.prev_hash,
        merkle_root: r.merkle_root, state_root: r.state_root,
        timestamp: r.timestamp, difficulty: r.difficulty, nonce: r.nonce,
        version: r.version, tx_count: r.tx_count, transactions: txs,
    }
}

// ── Transactions ──────────────────────────────────────────────────────────────

async fn get_txs_for_block(pool: &PgPool, block_height: i64) -> Result<Vec<TxSummary>, sqlx::Error> {
    let rows = sqlx::query_as::<_, TxRow>(
        "SELECT hash, block_height, block_hash, tx_index, sender, recipient, \
                amount, fee, nonce_val, gas_limit, gas_price, has_data, sig_type \
         FROM transactions WHERE block_height = $1 ORDER BY tx_index ASC"
    )
    .bind(block_height)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(TxSummary::from).collect())
}

pub async fn get_transaction(pool: &PgPool, hash: &str) -> Result<Option<TxSummary>, sqlx::Error> {
    let row = sqlx::query_as::<_, TxRow>(
        "SELECT hash, block_height, block_hash, tx_index, sender, recipient, \
                amount, fee, nonce_val, gas_limit, gas_price, has_data, sig_type \
         FROM transactions WHERE hash = $1"
    )
    .bind(hash)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(TxSummary::from))
}

pub async fn get_txs_by_address(pool: &PgPool, address: &str, limit: i64, offset: i64) -> Result<Vec<TxSummary>, sqlx::Error> {
    let rows = sqlx::query_as::<_, TxRow>(
        "SELECT hash, block_height, block_hash, tx_index, sender, recipient, \
                amount, fee, nonce_val, gas_limit, gas_price, has_data, sig_type \
         FROM transactions WHERE sender = $1 OR recipient = $1 \
         ORDER BY block_height DESC, tx_index ASC LIMIT $2 OFFSET $3"
    )
    .bind(address).bind(limit).bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(TxSummary::from).collect())
}

// ── Accounts ──────────────────────────────────────────────────────────────────

pub async fn get_account(
    pool: &PgPool,
    address: &str,
    tx_limit: i64,
    tx_offset: i64,
) -> Result<Option<AddressDetail>, sqlx::Error> {
    let acc = sqlx::query_as::<_, AccountRow>(
        "SELECT address, balance, nonce, updated_at FROM accounts WHERE address = $1"
    )
    .bind(address)
    .fetch_optional(pool)
    .await?;

    let txs = get_txs_by_address(pool, address, tx_limit, tx_offset).await?;

    Ok(acc.map(|a| AddressDetail {
        address:          a.address,
        balance:          a.balance,
        nonce:            a.nonce,
        updated_at_block: a.updated_at,
        transactions:     txs,
    }))
}

// ── Search ────────────────────────────────────────────────────────────────────

pub async fn search(pool: &PgPool, q: &str) -> Result<Vec<SearchResult>, sqlx::Error> {
    let mut results = Vec::new();

    // Block by hash (64-char hex)
    if let Ok(Some(row)) = sqlx::query_as::<_, HashRow>(
        "SELECT hash FROM blocks WHERE hash = $1 LIMIT 1"
    ).bind(q).fetch_optional(pool).await {
        results.push(SearchResult { kind: "block".into(), value: row.hash });
        return Ok(results);
    }

    // Block by height (numeric string)
    if let Ok(h) = q.parse::<i64>() {
        if let Ok(Some(row)) = sqlx::query_as::<_, HashRow>(
            "SELECT hash FROM blocks WHERE height = $1 LIMIT 1"
        ).bind(h).fetch_optional(pool).await {
            results.push(SearchResult { kind: "block".into(), value: row.hash });
            return Ok(results);
        }
    }

    // Transaction by hash
    if let Ok(Some(row)) = sqlx::query_as::<_, HashRow>(
        "SELECT hash FROM transactions WHERE hash = $1 LIMIT 1"
    ).bind(q).fetch_optional(pool).await {
        results.push(SearchResult { kind: "transaction".into(), value: row.hash });
        return Ok(results);
    }

    // Address
    if let Ok(Some(row)) = sqlx::query_as::<_, AddressRow>(
        "SELECT address FROM accounts WHERE address = $1 LIMIT 1"
    ).bind(q).fetch_optional(pool).await {
        results.push(SearchResult { kind: "address".into(), value: row.address });
        return Ok(results);
    }

    Ok(results)
}
