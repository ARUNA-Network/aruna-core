import { isCacheable } from '../services/cache';

/**
 * Cache middleware interface using Cloudflare caches.default.
 */

export async function matchCache(request: Request): Promise<Response | null> {
  const url = new URL(request.url);
  if (!isCacheable(request.method, url.pathname)) {
    return null;
  }
  const cache = (caches as any).default;
  return cache.match(request);
}

export async function putCache(request: Request, response: Response, ctx: { waitUntil: (p: Promise<any>) => void }): Promise<void> {
  const url = new URL(request.url);
  if (!isCacheable(request.method, url.pathname) || response.status !== 200) {
    return;
  }

  const cache = (caches as any).default;
  // Clone response and set max-age for 60 seconds
  const responseToCache = response.clone();
  const headers = new Headers(responseToCache.headers);
  headers.set('Cache-Control', 'public, max-age=60');
  
  const cacheableResponse = new Response(responseToCache.body, {
    status: responseToCache.status,
    statusText: responseToCache.statusText,
    headers,
  });

  ctx.waitUntil(cache.put(request, cacheableResponse));
}
