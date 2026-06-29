-- Migration 0001: Create blocks table
-- Stores one row per indexed block header.

CREATE TABLE IF NOT EXISTS blocks (
    height       BIGINT PRIMARY KEY,
    hash         TEXT   NOT NULL UNIQUE,
    prev_hash    TEXT   NOT NULL,
    merkle_root  TEXT   NOT NULL,
    state_root   TEXT   NOT NULL,
    timestamp    BIGINT NOT NULL,
    difficulty   BIGINT NOT NULL,
    nonce        BIGINT NOT NULL,
    version      INT    NOT NULL DEFAULT 1,
    tx_count     INT    NOT NULL DEFAULT 0,
    indexed_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS blocks_timestamp_idx ON blocks (timestamp DESC);
