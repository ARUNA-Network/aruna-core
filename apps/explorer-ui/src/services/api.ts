import { env } from '../config/env';
import { Block, Transaction, AddressData, Stats, NetworkData, SearchResult } from '../types';

async function apiFetch<T>(path: string): Promise<T> {
  const res = await fetch(env.API_BASE + path, {
    headers: { 'Accept': 'application/json' }
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body.error || `HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}

export const api = {
  stats: (): Promise<Stats> =>
    apiFetch<Stats>('/stats'),
  status: (): Promise<Stats> =>
    apiFetch<Stats>('/status'),
  blocks: (limit: number, offset: number): Promise<Block[]> =>
    apiFetch<Block[]>(`/blocks?limit=${limit}&offset=${offset}`),
  blockLatest: (): Promise<Block> =>
    apiFetch<Block>('/block/latest'),
  blockByHeight: (height: number): Promise<Block> =>
    apiFetch<Block>(`/block/${height}`),
  blockByHash: (hash: string): Promise<Block> =>
    apiFetch<Block>(`/block/hash/${hash}`),
  transaction: (hash: string): Promise<Transaction> =>
    apiFetch<Transaction>(`/transaction/${hash}`),
  address: (addr: string, limit: number, offset: number): Promise<AddressData> =>
    apiFetch<AddressData>(`/address/${addr}?limit=${limit}&offset=${offset}`),
  search: (q: string): Promise<SearchResult[]> =>
    apiFetch<SearchResult[]>(`/search?q=${encodeURIComponent(q)}`),
  network: (): Promise<NetworkData> =>
    apiFetch<NetworkData>('/network').catch(() => ({
      status: 'offline',
      peers: [],
      validators: { active_validators_count: 1, reward_address: "" }
    })),
};
