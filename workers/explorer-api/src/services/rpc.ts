import { Pool } from 'pg';

let poolInstance: Pool | null = null;

export function getDbPool(databaseUrl: string): Pool {
  if (!poolInstance) {
    poolInstance = new Pool({
      connectionString: databaseUrl,
      max: 10,
      idleTimeoutMillis: 30000,
      connectionTimeoutMillis: 2000,
    });
  }
  return poolInstance;
}

export interface BlockSummary {
  height: number;
  hash: string;
  prev_hash: string;
  merkle_root: string;
  state_root: string;
  timestamp: number;
  difficulty: number;
  nonce: number;
  version: number;
  tx_count: number;
}

export interface TxSummary {
  hash: string;
  block_height: number;
  block_hash: string;
  tx_index: number;
  sender: string;
  recipient: string;
  amount: string;
  fee: string;
  nonce_val: number;
  gas_limit: number;
  gas_price: number;
  has_data: boolean;
  sig_type: number;
}

export interface BlockDetail extends BlockSummary {
  transactions: TxSummary[];
}

export interface AddressDetail {
  address: string;
  balance: string;
  nonce: number;
  updated_at_block: number;
  transactions: TxSummary[];
}

export interface ChainStats {
  height: number;
  total_tx_count: number;
  best_hash: string;
  last_block_time: number;
}

export interface SearchResult {
  kind: 'block' | 'transaction' | 'address';
  value: string;
}

// ── DB Access Layer ──────────────────────────────────────────────────────────

export async function getChainStats(pool: Pool): Promise<ChainStats> {
  const res = await pool.query(
    'SELECT height, total_tx_count, best_hash, last_block_time FROM chain_stats WHERE id = 1'
  );
  if (res.rows.length === 0) {
    throw new Error('Chain stats not populated yet');
  }
  const row = res.rows[0];
  return {
    height: parseInt(row.height, 10),
    total_tx_count: parseInt(row.total_tx_count, 10),
    best_hash: row.best_hash,
    last_block_time: parseInt(row.last_block_time, 10),
  };
}

export async function getBlocks(pool: Pool, limit: number, offset: number): Promise<BlockSummary[]> {
  const res = await pool.query(
    `SELECT height, hash, prev_hash, merkle_root, state_root, timestamp, difficulty, nonce, version, tx_count 
     FROM blocks ORDER BY height DESC LIMIT $1 OFFSET $2`,
    [limit, offset]
  );
  return res.rows.map(row => ({
    height: parseInt(row.height, 10),
    hash: row.hash,
    prev_hash: row.prev_hash,
    merkle_root: row.merkle_root,
    state_root: row.state_root,
    timestamp: parseInt(row.timestamp, 10),
    difficulty: parseInt(row.difficulty, 10),
    nonce: parseInt(row.nonce, 10),
    version: row.version,
    tx_count: row.tx_count,
  }));
}

export async function countBlocks(pool: Pool): Promise<number> {
  const res = await pool.query('SELECT COUNT(*) AS cnt FROM blocks');
  return parseInt(res.rows[0].cnt, 10);
}

export async function getTxsForBlock(pool: Pool, blockHeight: number): Promise<TxSummary[]> {
  const res = await pool.query(
    `SELECT hash, block_height, block_hash, tx_index, sender, recipient, amount, fee, nonce_val, gas_limit, gas_price, has_data, sig_type 
     FROM transactions WHERE block_height = $1 ORDER BY tx_index ASC`,
    [blockHeight]
  );
  return res.rows.map(row => ({
    hash: row.hash,
    block_height: parseInt(row.block_height, 10),
    block_hash: row.block_hash,
    tx_index: row.tx_index,
    sender: row.sender,
    recipient: row.recipient,
    amount: row.amount,
    fee: row.fee,
    nonce_val: parseInt(row.nonce_val, 10),
    gas_limit: parseInt(row.gas_limit, 10),
    gas_price: parseInt(row.gas_price, 10),
    has_data: row.has_data,
    sig_type: row.sig_type,
  }));
}

export async function getBlockByHeight(pool: Pool, height: number): Promise<BlockDetail | null> {
  const res = await pool.query(
    `SELECT height, hash, prev_hash, merkle_root, state_root, timestamp, difficulty, nonce, version, tx_count 
     FROM blocks WHERE height = $1`,
    [height]
  );
  if (res.rows.length === 0) return null;
  const block = res.rows[0];
  const txs = await getTxsForBlock(pool, height);
  return {
    height: parseInt(block.height, 10),
    hash: block.hash,
    prev_hash: block.prev_hash,
    merkle_root: block.merkle_root,
    state_root: block.state_root,
    timestamp: parseInt(block.timestamp, 10),
    difficulty: parseInt(block.difficulty, 10),
    nonce: parseInt(block.nonce, 10),
    version: block.version,
    tx_count: block.tx_count,
    transactions: txs,
  };
}

export async function getBlockByHash(pool: Pool, hash: string): Promise<BlockDetail | null> {
  const res = await pool.query(
    `SELECT height, hash, prev_hash, merkle_root, state_root, timestamp, difficulty, nonce, version, tx_count 
     FROM blocks WHERE hash = $1`,
    [hash]
  );
  if (res.rows.length === 0) return null;
  const block = res.rows[0];
  const height = parseInt(block.height, 10);
  const txs = await getTxsForBlock(pool, height);
  return {
    height,
    hash: block.hash,
    prev_hash: block.prev_hash,
    merkle_root: block.merkle_root,
    state_root: block.state_root,
    timestamp: parseInt(block.timestamp, 10),
    difficulty: parseInt(block.difficulty, 10),
    nonce: parseInt(block.nonce, 10),
    version: block.version,
    tx_count: block.tx_count,
    transactions: txs,
  };
}

export async function getLatestBlock(pool: Pool): Promise<BlockDetail | null> {
  const res = await pool.query(
    `SELECT height, hash, prev_hash, merkle_root, state_root, timestamp, difficulty, nonce, version, tx_count 
     FROM blocks ORDER BY height DESC LIMIT 1`
  );
  if (res.rows.length === 0) return null;
  const block = res.rows[0];
  const height = parseInt(block.height, 10);
  const txs = await getTxsForBlock(pool, height);
  return {
    height,
    hash: block.hash,
    prev_hash: block.prev_hash,
    merkle_root: block.merkle_root,
    state_root: block.state_root,
    timestamp: parseInt(block.timestamp, 10),
    difficulty: parseInt(block.difficulty, 10),
    nonce: parseInt(block.nonce, 10),
    version: block.version,
    tx_count: block.tx_count,
    transactions: txs,
  };
}

export async function getTransaction(pool: Pool, hash: string): Promise<TxSummary | null> {
  const res = await pool.query(
    `SELECT hash, block_height, block_hash, tx_index, sender, recipient, amount, fee, nonce_val, gas_limit, gas_price, has_data, sig_type 
     FROM transactions WHERE hash = $1`,
    [hash]
  );
  if (res.rows.length === 0) return null;
  const row = res.rows[0];
  return {
    hash: row.hash,
    block_height: parseInt(row.block_height, 10),
    block_hash: row.block_hash,
    tx_index: row.tx_index,
    sender: row.sender,
    recipient: row.recipient,
    amount: row.amount,
    fee: row.fee,
    nonce_val: parseInt(row.nonce_val, 10),
    gas_limit: parseInt(row.gas_limit, 10),
    gas_price: parseInt(row.gas_price, 10),
    has_data: row.has_data,
    sig_type: row.sig_type,
  };
}

export async function getTxsByAddress(pool: Pool, address: string, limit: number, offset: number): Promise<TxSummary[]> {
  const res = await pool.query(
    `SELECT hash, block_height, block_hash, tx_index, sender, recipient, amount, fee, nonce_val, gas_limit, gas_price, has_data, sig_type 
     FROM transactions WHERE sender = $1 OR recipient = $1 
     ORDER BY block_height DESC, tx_index ASC LIMIT $2 OFFSET $3`,
    [address, limit, offset]
  );
  return res.rows.map(row => ({
    hash: row.hash,
    block_height: parseInt(row.block_height, 10),
    block_hash: row.block_hash,
    tx_index: row.tx_index,
    sender: row.sender,
    recipient: row.recipient,
    amount: row.amount,
    fee: row.fee,
    nonce_val: parseInt(row.nonce_val, 10),
    gas_limit: parseInt(row.gas_limit, 10),
    gas_price: parseInt(row.gas_price, 10),
    has_data: row.has_data,
    sig_type: row.sig_type,
  }));
}

export async function getAccount(pool: Pool, address: string, txLimit: number, txOffset: number): Promise<AddressDetail | null> {
  const res = await pool.query(
    'SELECT address, balance, nonce, updated_at FROM accounts WHERE address = $1',
    [address]
  );
  if (res.rows.length === 0) return null;
  const acc = res.rows[0];
  const txs = await getTxsByAddress(pool, address, txLimit, txOffset);
  return {
    address: acc.address,
    balance: acc.balance,
    nonce: parseInt(acc.nonce, 10),
    updated_at_block: parseInt(acc.updated_at, 10),
    transactions: txs,
  };
}

export async function search(pool: Pool, q: string): Promise<SearchResult[]> {
  const results: SearchResult[] = [];

  // Check block hash (64 hex characters)
  const blockHashRes = await pool.query('SELECT hash FROM blocks WHERE hash = $1 LIMIT 1', [q]);
  if (blockHashRes.rows.length > 0) {
    results.push({ kind: 'block', value: blockHashRes.rows[0].hash });
    return results;
  }

  // Check block height
  const heightVal = parseInt(q, 10);
  if (!isNaN(heightVal)) {
    const blockHeightRes = await pool.query('SELECT hash FROM blocks WHERE height = $1 LIMIT 1', [heightVal]);
    if (blockHeightRes.rows.length > 0) {
      results.push({ kind: 'block', value: blockHeightRes.rows[0].hash });
      return results;
    }
  }

  // Check transaction
  const txRes = await pool.query('SELECT hash FROM transactions WHERE hash = $1 LIMIT 1', [q]);
  if (txRes.rows.length > 0) {
    results.push({ kind: 'transaction', value: txRes.rows[0].hash });
    return results;
  }

  // Check address
  const addrRes = await pool.query('SELECT address FROM accounts WHERE address = $1 LIMIT 1', [q]);
  if (addrRes.rows.length > 0) {
    results.push({ kind: 'address', value: addrRes.rows[0].address });
    return results;
  }

  return results;
}

// ── Node RPC Access Layer ───────────────────────────────────────────────────

export async function fetchNodeRpc(nodeUrl: string, pathname: string): Promise<any> {
  const res = await fetch(`${nodeUrl}${pathname}`);
  if (!res.ok) {
    throw new Error(`Node RPC responded with status ${res.status}`);
  }
  return res.json();
}
