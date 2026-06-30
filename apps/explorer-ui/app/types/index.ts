export interface Block {
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
  transactions?: Transaction[];
}

export interface Transaction {
  hash: string;
  block_height: number;
  block_hash: string;
  tx_index: number;
  sender: string;
  recipient: string;
  amount: number;
  fee: number;
  nonce_val: number;
  gas_limit: number;
  gas_price: number;
  sig_type: number;
  has_data: boolean;
}

export interface AddressData {
  balance: number;
  nonce: number;
  updated_at_block: number;
  transactions: Transaction[];
}

export interface Stats {
  height: number;
  total_tx_count: number;
  best_hash: string;
  last_block_time: number;
  node?: {
    network: string;
    version: string;
    chain_id: number;
    height: number;
    best_block: string;
    peer_count: number;
    uptime_seconds: number;
    synced: boolean;
  };
}

export interface NetworkData {
  status: string;
  peers: string[];
  validators: {
    active_validators_count: number;
    reward_address: string;
  };
}

export interface SearchResult {
  kind: 'block' | 'transaction' | 'address';
  value: string;
}
