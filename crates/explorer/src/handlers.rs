//! Axum route handlers for the ARUNA Explorer REST API.
//!
//! All handlers read from PostgreSQL via the `db` module.
//! No RocksDB access. No Node RPC calls.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use crate::{db, models::*};

pub type AppState = PgPool;

// ── Pagination params ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl PaginationParams {
    fn limit(&self) -> i64  { self.limit.unwrap_or(20).clamp(1, 100) }
    fn offset(&self) -> i64 { self.offset.unwrap_or(0).max(0) }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn not_found(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": msg })))
}

fn server_err(e: sqlx::Error) -> (StatusCode, Json<Value>) {
    tracing::error!("DB error: {}", e);
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Database error" })))
}

// ── /api/v1/stats ─────────────────────────────────────────────────────────────

pub async fn get_stats(
    State(pool): State<AppState>,
) -> Result<Json<ChainStats>, (StatusCode, Json<Value>)> {
    db::get_chain_stats(&pool).await
        .map(Json)
        .map_err(server_err)
}

// ── /api/v1/blocks ────────────────────────────────────────────────────────────

pub async fn get_blocks(
    State(pool): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Paginated<BlockSummary>>, (StatusCode, Json<Value>)> {
    let limit  = params.limit();
    let offset = params.offset();
    let items = db::get_blocks(&pool, limit, offset).await.map_err(server_err)?;
    let total = db::count_blocks(&pool).await.map_err(server_err)?;
    Ok(Json(Paginated { items, total: Some(total), limit, offset }))
}

// ── /api/v1/block/latest ─────────────────────────────────────────────────────

pub async fn get_block_latest(
    State(pool): State<AppState>,
) -> Result<Json<BlockDetail>, (StatusCode, Json<Value>)> {
    db::get_latest_block(&pool).await
        .map_err(server_err)?
        .map(Json)
        .ok_or_else(|| not_found("No blocks indexed yet"))
}

// ── /api/v1/block/height/:n ───────────────────────────────────────────────────

pub async fn get_block_by_height(
    State(pool): State<AppState>,
    Path(n): Path<i64>,
) -> Result<Json<BlockDetail>, (StatusCode, Json<Value>)> {
    db::get_block_by_height(&pool, n).await
        .map_err(server_err)?
        .map(Json)
        .ok_or_else(|| not_found(&format!("Block at height {} not found", n)))
}

// ── /api/v1/block/hash/:hash ──────────────────────────────────────────────────

pub async fn get_block_by_hash(
    State(pool): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<BlockDetail>, (StatusCode, Json<Value>)> {
    db::get_block_by_hash(&pool, &hash).await
        .map_err(server_err)?
        .map(Json)
        .ok_or_else(|| not_found("Block not found"))
}

// ── /api/v1/transaction/:hash ─────────────────────────────────────────────────

pub async fn get_transaction(
    State(pool): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<TxSummary>, (StatusCode, Json<Value>)> {
    db::get_transaction(&pool, &hash).await
        .map_err(server_err)?
        .map(Json)
        .ok_or_else(|| not_found("Transaction not found"))
}

// ── /api/v1/address/:addr ─────────────────────────────────────────────────────

pub async fn get_address(
    State(pool): State<AppState>,
    Path(addr): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<AddressDetail>, (StatusCode, Json<Value>)> {
    let limit  = params.limit();
    let offset = params.offset();
    db::get_account(&pool, &addr, limit, offset).await
        .map_err(server_err)?
        .map(Json)
        .ok_or_else(|| not_found("Address not found"))
}

// ── /api/v1/search?q= ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
}

pub async fn search(
    State(pool): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, Json<Value>)> {
    let q = params.q.as_deref().unwrap_or("").trim().to_string();
    if q.is_empty() {
        return Ok(Json(vec![]));
    }
    db::search(&pool, &q).await
        .map(Json)
        .map_err(server_err)
}

// ── /health ───────────────────────────────────────────────────────────────────

pub async fn health() -> Json<Value> {
    Json(serde_json::json!({ "status": "healthy" }))
}
