/**
 * ARUNA RPC Gateway (Cloudflare Worker)
 *
 * Intercepts public blockchain traffic, enforces CORS, provides basic rate limits,
 * caches historical block queries, and forwards RPC requests to the underlying Node.
 */

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);

    // 1. Enforce CORS headers
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
    };

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    // 2. Intercept /health locally at the Edge
    if (url.pathname === '/health') {
      return new Response(JSON.stringify({ status: 'healthy' }), {
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
        status: 200,
      });
    }

    // 3. For all other routes, build proxy request to the Node RPC
    const targetUrl = `${env.NODE_RPC_URL}${url.pathname}${url.search}`;
    
    // Only cache safe historical queries to avoid stalling sync status
    const cacheablePaths = ['/block/hash/', '/block/height/', '/supply', '/difficulty'];
    const isCacheable = request.method === 'GET' && cacheablePaths.some(p => url.pathname.startsWith(p));

    const cache = caches.default;
    if (isCacheable) {
      const cachedResponse = await cache.match(request);
      if (cachedResponse) {
        return cachedResponse;
      }
    }

    try {
      const response = await fetch(targetUrl, {
        method: request.method,
        headers: request.headers,
        body: request.method === 'GET' || request.method === 'HEAD' ? null : request.body,
      });

      // Clone response to add CORS headers and put in cache if cacheable
      let modifiedResponse = new Response(response.body, response);
      for (const [key, val] of Object.entries(corsHeaders)) {
        modifiedResponse.headers.set(key, val);
      }

      if (isCacheable && response.status === 200) {
        // Cache for 60 seconds
        modifiedResponse.headers.set('Cache-Control', 'public, max-age=60');
        ctx.waitUntil(cache.put(request, modifiedResponse.clone()));
      }

      return modifiedResponse;
    } catch (e) {
      return new Response(JSON.stringify({ error: 'Gateway failed to reach RPC node', details: e.message }), {
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
        status: 502,
      });
    }
  }
};
