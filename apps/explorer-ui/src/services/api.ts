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

export async function getStatus(): Promise<Stats> {
  return apiFetch<Stats>('/status');
}

export async function getLatestBlock(): Promise<Block> {
  return apiFetch<Block>('/block/latest');
}

export async function getBlock(heightOrHash: string | number): Promise<Block> {
  if (typeof heightOrHash === 'number' || /^\d+$/.test(String(heightOrHash))) {
    return apiFetch<Block>(`/block/${heightOrHash}`);
  }
  return apiFetch<Block>(`/block/hash/${heightOrHash}`);
}

export async function getTransaction(hash: string): Promise<Transaction> {
  return apiFetch<Transaction>(`/transaction/${hash}`);
}

export async function getAddress(addr: string, limit = 20, offset = 0): Promise<AddressData> {
  return apiFetch<AddressData>(`/address/${addr}?limit=${limit}&offset=${offset}`);
}

export async function getBlocks(limit: number, offset: number): Promise<Block[]> {
  return apiFetch<Block[]>(`/blocks?limit=${limit}&offset=${offset}`);
}

export async function getNetwork(): Promise<NetworkData> {
  return apiFetch<NetworkData>('/network').catch(() => ({
    status: 'offline',
    peers: [],
    validators: { active_validators_count: 1, reward_address: "" }
  }));
}

export async function search(q: string): Promise<SearchResult[]> {
  return apiFetch<SearchResult[]>(`/search?q=${encodeURIComponent(q)}`);
}
