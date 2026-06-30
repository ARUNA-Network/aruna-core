import { useRuntimeConfig } from '#app'
import type { Block, Transaction, AddressData, Stats, NetworkData, SearchResult } from '../types'

function getApiBase(): string {
  try {
    const config = useRuntimeConfig()
    return config.public.apiBase
  } catch {
    return 'https://api.jojowi.web.id/api/v1'
  }
}

async function apiFetch<T>(path: string): Promise<T> {
  const apiBase = getApiBase()
  return $fetch<T>(apiBase + path, {
    headers: { 'Accept': 'application/json' }
  })
}

export async function getStatus(): Promise<Stats> {
  return apiFetch<Stats>('/status')
}

export async function getLatestBlock(): Promise<Block> {
  return apiFetch<Block>('/block/latest')
}

export async function getBlock(heightOrHash: string | number): Promise<Block> {
  if (typeof heightOrHash === 'number' || /^\d+$/.test(String(heightOrHash))) {
    return apiFetch<Block>(`/block/${heightOrHash}`)
  }
  return apiFetch<Block>(`/block/hash/${heightOrHash}`)
}

export async function getTransaction(hash: string): Promise<Transaction> {
  return apiFetch<Transaction>(`/transaction/${hash}`)
}

export async function getAddress(addr: string, limit = 20, offset = 0): Promise<AddressData> {
  return apiFetch<AddressData>(`/address/${addr}?limit=${limit}&offset=${offset}`)
}

export async function getBlocks(limit: number, offset: number): Promise<Block[]> {
  return apiFetch<Block[]>(`/blocks?limit=${limit}&offset=${offset}`)
}

export async function getNetwork(): Promise<NetworkData> {
  return apiFetch<NetworkData>('/network').catch(() => ({
    status: 'offline',
    peers: [],
    validators: { active_validators_count: 1, reward_address: "" }
  }))
}

export async function search(q: string): Promise<SearchResult[]> {
  return apiFetch<SearchResult[]>(`/search?q=${encodeURIComponent(q)}`)
}
