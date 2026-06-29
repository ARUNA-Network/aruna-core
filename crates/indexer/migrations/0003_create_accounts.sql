-- Migration 0003: Create accounts table
-- Stores the last-known balance and nonce for each address.
-- Updated whenever an account appears in a transaction or block reward.

CREATE TABLE IF NOT EXISTS accounts (
    address    TEXT   PRIMARY KEY,
    balance    BIGINT NOT NULL DEFAULT 0,
    nonce      BIGINT NOT NULL DEFAULT 0,
    updated_at BIGINT NOT NULL DEFAULT 0   -- block height of last update
);
