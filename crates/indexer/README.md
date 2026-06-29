# aruna-indexer

ARUNA Network block indexer — polls the Node RPC and writes to PostgreSQL for Explorer consumption.

## Architecture

See [ADR-0017](../../docs/adr/adr-0017-explorer-architecture.md).

```
Node RPC (port 8080)
    │ HTTP polling
    ▼
aruna-indexer
    │ SQL writes
    ▼
PostgreSQL (aruna_explorer DB)
    │ SQL reads
    ▼
aruna-explorer REST API (port 3000)
    │ JSON
    ▼
Explorer UI
```

## Configuration

`config/indexer.toml`:

```toml
[indexer]
node_rpc_url       = "http://127.0.0.1:8080"
database_url       = "postgres://aruna:aruna@localhost:5432/aruna_explorer"
poll_interval_secs = 5
```

## Running

```bash
# From workspace root
cargo run -p aruna-indexer

# Custom config
cargo run -p aruna-indexer -- --config /path/to/indexer.toml
```

## Database Schema

Migrations are embedded and run automatically on startup via `sqlx::migrate!`.

| Table         | Purpose                                      |
|---------------|----------------------------------------------|
| `blocks`      | One row per indexed block header              |
| `transactions`| One row per confirmed transaction             |
| `accounts`    | Last-known balance/nonce per address          |
| `chain_stats` | Singleton aggregate (height, tx count, etc.) |

## Environment Variables

| Variable   | Default | Description                    |
|------------|---------|--------------------------------|
| `RUST_LOG` | `info`  | Log level (tracing subscriber) |
