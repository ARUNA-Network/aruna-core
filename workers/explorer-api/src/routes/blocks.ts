import { Pool } from 'pg';
import { getBlocks, countBlocks, getLatestBlock, getBlockByHeight, getBlockByHash } from '../services/rpc';

export async function handleBlocks(request: Request, pool: Pool): Promise<Response> {
  const url = new URL(request.url);

  // 1. Paginated blocks list
  if (url.pathname === '/api/v1/blocks') {
    const limit = Math.min(Math.max(parseInt(url.searchParams.get('limit') || '20', 10), 1), 100);
    const offset = Math.max(parseInt(url.searchParams.get('offset') || '0', 10), 0);

    try {
      const items = await getBlocks(pool, limit, offset);
      const total = await countBlocks(pool);

      return new Response(JSON.stringify({
        items,
        total,
        limit,
        offset
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    } catch (err) {
      return new Response(JSON.stringify({ error: (err as Error).message }), {
        status: 500,
        headers: { 'Content-Type': 'application/json' },
      });
    }
  }

  // 2. Latest block
  if (url.pathname === '/api/v1/block/latest') {
    try {
      const block = await getLatestBlock(pool);
      if (!block) {
        return new Response(JSON.stringify({ error: 'No blocks indexed yet' }), {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
        });
      }
      return new Response(JSON.stringify(block), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    } catch (err) {
      return new Response(JSON.stringify({ error: (err as Error).message }), {
        status: 500,
        headers: { 'Content-Type': 'application/json' },
      });
    }
  }

  // 3. Block by height
  const heightMatch = url.pathname.match(/^\/api\/v1\/block\/height\/(\d+)$/);
  if (heightMatch) {
    const height = parseInt(heightMatch[1], 10);
    try {
      const block = await getBlockByHeight(pool, height);
      if (!block) {
        return new Response(JSON.stringify({ error: `Block at height ${height} not found` }), {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
        });
      }
      return new Response(JSON.stringify(block), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    } catch (err) {
      return new Response(JSON.stringify({ error: (err as Error).message }), {
        status: 500,
        headers: { 'Content-Type': 'application/json' },
      });
    }
  }

  // 4. Block by hash
  const hashMatch = url.pathname.match(/^\/api\/v1\/block\/hash\/([a-fA-F0-9]+)$/);
  if (hashMatch) {
    const hash = hashMatch[1];
    try {
      const block = await getBlockByHash(pool, hash);
      if (!block) {
        return new Response(JSON.stringify({ error: `Block with hash ${hash} not found` }), {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
        });
      }
      return new Response(JSON.stringify(block), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    } catch (err) {
      return new Response(JSON.stringify({ error: (err as Error).message }), {
        status: 500,
        headers: { 'Content-Type': 'application/json' },
      });
    }
  }

  return new Response(JSON.stringify({ error: 'Route not found' }), {
    status: 404,
    headers: { 'Content-Type': 'application/json' },
  });
}
