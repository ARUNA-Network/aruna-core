-- Migration 0004: Create chain_stats table
-- Singleton row (id = 1) tracking global chain statistics.

CREATE TABLE IF NOT EXISTS chain_stats (
    id              INT  PRIMARY KEY DEFAULT 1,
    height          BIGINT NOT NULL DEFAULT 0,
    total_tx_count  BIGINT NOT NULL DEFAULT 0,
    best_hash       TEXT   NOT NULL DEFAULT '',
    last_block_time BIGINT NOT NULL DEFAULT 0,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chain_stats_singleton CHECK (id = 1)
);

-- Seed the singleton row so UPDATE statements always find a row.
INSERT INTO chain_stats (id, height, total_tx_count, best_hash, last_block_time)
VALUES (1, 0, 0, '', 0)
ON CONFLICT (id) DO NOTHING;
