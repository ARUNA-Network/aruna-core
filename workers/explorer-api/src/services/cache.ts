/**
 * Cache utilities for the Explorer Edge Worker.
 */

export function buildCacheKey(url: string): string {
  const parsed = new URL(url);
  return `${parsed.pathname}${parsed.search}`;
}

export function isCacheable(method: string, pathname: string): boolean {
  if (method !== 'GET') return false;
  
  const cacheablePrefixes = [
    '/api/v1/block/hash/',
    '/api/v1/block/height/',
    '/api/v1/transaction/',
    '/api/v1/stats'
  ];

  return cacheablePrefixes.some(prefix => pathname.startsWith(prefix));
}
