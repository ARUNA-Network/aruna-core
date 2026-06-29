import { getDbPool, fetchNodeRpc } from './services/rpc';
import { handleOptions, applyCors } from './middleware/cors';
import { logRequest } from './middleware/logger';
import { matchCache, putCache } from './middleware/cache';
import { handleStatus } from './routes/status';
import { handleBlocks } from './routes/blocks';
import { handleTransactions } from './routes/transactions';
import { handleAddresses } from './routes/addresses';
import { handleSearch } from './routes/search';

interface Env {
  DATABASE_URL: string;
  NODE_RPC_URL: string;
}

export default {
  async fetch(request: Request, env: Env, ctx: { waitUntil: (p: Promise<any>) => void }): Promise<Response> {
    const startMs = Date.now();
    const url = new URL(request.url);

    // 1. Handle CORS Preflight Preflight OPTIONS request
    const optionsRes = handleOptions(request);
    if (optionsRes) return optionsRes;

    // 2. Lookup Cache for Cacheable GET requests
    const cachedResponse = await matchCache(request);
    if (cachedResponse) {
      logRequest(request.method, url.pathname, startMs, 200);
      return applyCors(cachedResponse);
    }

    // 3. Connect to PostgreSQL Pool
    let pool;
    try {
      pool = getDbPool(env.DATABASE_URL);
    } catch (e) {
      const errRes = new Response(JSON.stringify({ error: 'DB Pool Error', details: (e as Error).message }), {
        status: 500,
        headers: { 'Content-Type': 'application/json' },
      });
      logRequest(request.method, url.pathname, startMs, 500);
      return applyCors(errRes);
    }

    let response: Response;

    // 4. Route Distribution
    if (url.pathname === '/api/v1/stats' || url.pathname === '/api/v1/status') {
      response = await handleStatus(request, pool, env);
    } else if (
      url.pathname === '/api/v1/blocks' ||
      url.pathname === '/api/v1/block/latest' ||
      /^\/api\/v1\/block\/\d+$/.test(url.pathname) ||
      url.pathname.startsWith('/api/v1/block/hash/')
    ) {
      response = await handleBlocks(request, pool);
    } else if (url.pathname.startsWith('/api/v1/transaction/')) {
      response = await handleTransactions(request, pool);
    } else if (url.pathname.startsWith('/api/v1/address/')) {
      response = await handleAddresses(request, pool);
    } else if (url.pathname === '/api/v1/search') {
      response = await handleSearch(request, pool);
    } else if (url.pathname === '/api/v1/network') {
      try {
        const [peers, validators] = await Promise.all([
          fetchNodeRpc(env.NODE_RPC_URL, '/peers').catch(() => ({ peers: [] })),
          fetchNodeRpc(env.NODE_RPC_URL, '/validators').catch(() => ({ active_validators_count: 1, reward_address: "" })),
        ]);
        response = new Response(JSON.stringify({
          status: 'healthy',
          peers: peers.peers || [],
          validators: validators,
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        });
      } catch (err) {
        response = new Response(JSON.stringify({
          status: 'offline',
          peers: [],
          validators: { active_validators_count: 1, reward_address: "" }
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        });
      }
    } else if (url.pathname === '/health') {
      response = new Response(JSON.stringify({ status: 'healthy' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    } else {
      response = new Response(JSON.stringify({ error: 'Route not found' }), {
        status: 404,
        headers: { 'Content-Type': 'application/json' },
      });
    }

    // 5. Apply Logger Middleware
    logRequest(request.method, url.pathname, startMs, response.status);

    // 6. Put in cache if cacheable
    const corsResponse = applyCors(response);
    await putCache(request, corsResponse, ctx);

    return corsResponse;
  }
};
