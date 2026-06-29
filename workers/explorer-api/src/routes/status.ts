import { Pool } from 'pg';
import { getChainStats, fetchNodeRpc } from '../services/rpc';

export async function handleStatus(request: Request, pool: Pool, env: { NODE_RPC_URL: string }): Promise<Response> {
  const url = new URL(request.url);

  try {
    // 1. Get chain stats from database
    const dbStats = await getChainStats(pool);

    // 2. Fetch live metrics from node if available (fallback cleanly if offline)
    let liveStats: any = null;
    try {
      liveStats = await fetchNodeRpc(env.NODE_RPC_URL, '/status');
    } catch (e) {
      console.warn('Node RPC offline, serving DB stats only:', (e as Error).message);
    }

    const payload = {
      height: dbStats.height,
      total_tx_count: dbStats.total_tx_count,
      best_hash: dbStats.best_hash,
      last_block_time: dbStats.last_block_time,
      node: liveStats ? {
        version: liveStats.version,
        network: liveStats.network,
        synced: liveStats.synced,
        peer_count: liveStats.peer_count,
        uptime_seconds: liveStats.uptime_seconds,
      } : null,
    };

    return new Response(JSON.stringify(payload), {
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
