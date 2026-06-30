import { Pool } from 'pg';
import { getTransaction } from '../services/rpc';

export async function handleTransactions(request: Request, pool: Pool): Promise<Response> {
  const url = new URL(request.url);

  const hashMatch = url.pathname.match(/^\/(?:api|explorer)\/v1\/transaction\/([a-fA-F0-9]+)$/);
  if (!hashMatch) {
    return new Response(JSON.stringify({ error: 'Route not found' }), {
      status: 404,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const hash = hashMatch[1];
  try {
    const tx = await getTransaction(pool, hash);
    if (!tx) {
      return new Response(JSON.stringify({ error: 'Transaction not found' }), {
        status: 404,
        headers: { 'Content-Type': 'application/json' },
      });
    }
    return new Response(JSON.stringify(tx), {
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
