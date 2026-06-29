# ADR-0017: Explorer Architecture — Indexer → REST API → Explorer UI

## Status
Accepted

## Context

Phase 4 of the ARUNA Network roadmap introduces the Block Explorer. The explorer must allow community members and validators to inspect blocks, transactions, and accounts on the public testnet.

There are multiple ways to implement a block explorer:

1. **Direct RocksDB access**: The explorer binary opens RocksDB in read-only mode and serves data directly.
2. **Node RPC proxy**: The explorer proxies requests to the node's internal RPC.
3. **Indexer + separate read database**: A background service polls the node RPC and writes to a read-optimized SQL database; the explorer reads from SQL.

Option 1 was briefly prototyped (`crates/explorer/src/main.rs`, pre-ADR-0017) but rejected because it introduces tight coupling between the explorer and the node's internal storage format. Any storage layer change breaks the explorer.

Option 2 is rejected because it creates load on the node's RPC under high explorer traffic, and the node RPC is not designed for pagination or historical analytics queries.

## Decision

We adopt **Option 3**: a three-tier architecture composed of:

```
ARUNA Node (RocksDB internal)
    │
    │ (node internal reads only)
    ▼
Node RPC — HTTP, port 8080, localhost-only
    │
    │ HTTP polling (internal network, never public)
    ▼
aruna-indexer — new crate (crates/indexer/)
    │
    │ SQL writes
    ▼
PostgreSQL — aruna_explorer database
    │
    │ SQL reads (read-heavy, analytics-optimized)
    ▼
aruna-explorer REST API — HTTP, port 3000, internal (behind Cloudflare)
    │
    │ JSON /api/v1/* (public via Cloudflare CDN)
    ▼
Explorer UI — static HTML/CSS/JS (apps/explorer/)
```

### Indexer (`crates/indexer/`)

- Runs as an independent binary (`aruna-indexer`)
- Polls the Node RPC at a configurable interval (default: 5 seconds)
- Compares its local indexed height to the node's canonical height
- For each new block: fetches block data, transactions, and relevant account states from the Node RPC; writes to PostgreSQL using atomic transactions
- Configuration via `config/indexer.toml`
- Never writes to or reads from RocksDB directly

### REST API (`crates/explorer/`)

- Complete rewrite of the prototype binary
- Runs as `aruna-explorer`
- Reads exclusively from PostgreSQL (`sqlx` async driver)
- Exposes `/api/v1/*` namespace to avoid path conflicts
- Implements pagination, search, and aggregate stats from indexed data
- Configuration via `config/explorer.toml`
- CORS headers enabled for browser access

### Explorer UI (`apps/explorer/`)

- Pure static files: HTML5, Vanilla CSS, JavaScript (no framework)
- Reads exclusively from the REST API (`/api/v1/*`)
- Can be served by any static file server or CDN
- Supports: Dashboard, Block detail, Transaction detail, Address detail, Search

## Consequences

### Positive
- **Node isolation**: Explorer traffic does not impact node performance or consensus.
- **Storage decoupling**: Explorer is not tied to RocksDB layout or schema changes.
- **Query power**: PostgreSQL supports complex pagination, aggregation, and full-text search that RocksDB cannot.
- **Horizontal scalability**: Explorer API can be scaled independently of the node.
- **Replay resilience**: Indexer can re-index from the node RPC if the PostgreSQL database is lost.
- **Security**: Node RPC and RocksDB are never directly exposed to the public.

### Negative
- **Eventual consistency**: Indexed data is slightly behind the node tip (by 1 polling interval, ~5 seconds). Acceptable for an explorer.
- **PostgreSQL dependency**: Adds operational overhead (database provisioning, backup).
- **Re-indexing time**: On fresh deployments, the indexer must catch up from block 0 to current tip.

## Schema

See `crates/indexer/migrations/` for the full PostgreSQL schema:
- `blocks` — one row per indexed block
- `transactions` — one row per confirmed transaction
- `accounts` — last known balance/nonce per address
- `chain_stats` — singleton aggregate row

## References
- ADR-0002: Monorepo Strategy
- ADR-0004: Account-Based Model
- ADR-0012: Node Types
- Node RPC implementation: `crates/node/src/rpc.rs`
- Indexer crate: `crates/indexer/`
- Explorer API crate: `crates/explorer/`
- Explorer UI: `apps/explorer/`
