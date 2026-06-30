import { Pool } from 'pg';
import { getAccount } from '../services/rpc';

export async function handleAddresses(request: Request, pool: Pool): Promise<Response> {
  const url = new URL(request.url);

  const addrMatch = url.pathname.match(/^\/(?:api|explorer)\/v1\/address\/([a-zA-Z0-9]+)$/);
  if (!addrMatch) {
    return new Response(JSON.stringify({ error: 'Route not found' }), {
      status: 404,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const address = addrMatch[1];
  const limit = Math.min(Math.max(parseInt(url.searchParams.get('limit') || '20', 10), 1), 100);
  const offset = Math.max(parseInt(url.searchParams.get('offset') || '0', 10), 0);

  try {
    const acc = await getAccount(pool, address, limit, offset);
    if (!acc) {
      return new Response(JSON.stringify({ error: 'Address not found' }), {
        status: 404,
        headers: { 'Content-Type': 'application/json' },
      });
    }
    return new Response(JSON.stringify(acc), {
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
