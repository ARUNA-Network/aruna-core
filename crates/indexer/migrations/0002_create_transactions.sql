-- Migration 0002: Create transactions table
-- Stores one row per confirmed transaction with sender/recipient decoded.

CREATE TABLE IF NOT EXISTS transactions (
    hash         TEXT   PRIMARY KEY,
    block_height BIGINT NOT NULL REFERENCES blocks(height) ON DELETE CASCADE,
    block_hash   TEXT   NOT NULL,
    tx_index     INT    NOT NULL,
    sender       TEXT   NOT NULL,
    recipient    TEXT   NOT NULL,
    amount       BIGINT NOT NULL,
    fee          BIGINT NOT NULL,
    nonce_val    BIGINT NOT NULL,
    gas_limit    BIGINT NOT NULL,
    gas_price    BIGINT NOT NULL,
    has_data     BOOLEAN NOT NULL DEFAULT FALSE,
    sig_type     SMALLINT NOT NULL DEFAULT 0,
    indexed_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS txs_block_height_idx ON transactions (block_height DESC);
CREATE INDEX IF NOT EXISTS txs_sender_idx       ON transactions (sender);
CREATE INDEX IF NOT EXISTS txs_recipient_idx    ON transactions (recipient);
