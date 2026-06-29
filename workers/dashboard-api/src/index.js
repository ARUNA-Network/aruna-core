/**
 * ARUNA Dashboard API (Cloudflare Worker)
 *
 * Telemetry proxy that fetches plain-text Prometheus metrics and status from the node,
 * parses them into structured JSON, and returns it to apps/dashboard.
 */

export default {
  async fetch(request, env) {
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
    };

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    try {
      // 1. Fetch `/status` and `/metrics` from the Node RPC
      const [statusRes, metricsRes] = await Promise.all([
        fetch(`${env.NODE_RPC_URL}/status`),
        fetch(`${env.NODE_RPC_URL}/metrics`)
      ]);

      if (!statusRes.ok || !metricsRes.ok) {
        throw new Error('Failed to fetch from Node RPC');
      }

      const status = await statusRes.json();
      const rawMetrics = await metricsRes.text();

      // 2. Parse Prometheus plain-text metrics into a key-value object
      const metrics = {};
      const lines = rawMetrics.split('\n');
      for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed && !trimmed.startsWith('#')) {
          const parts = trimmed.split(' ');
          if (parts.length === 2) {
            metrics[parts[0]] = parseFloat(parts[1]);
          }
        }
      }

      // 3. Construct the telemetry JSON response
      const telemetry = {
        node: {
          network: status.network,
          version: status.version,
          chain_id: status.chain_id,
          synced: status.synced,
          uptime_seconds: status.uptime_seconds,
          peer_count: status.peer_count
        },
        metrics: {
          block_height: metrics['aruna_block_height'] || 0,
          mempool_size: metrics['aruna_mempool_size'] || 0,
          fork_count: metrics['aruna_fork_count'] || 0,
          rpc_requests_total: metrics['aruna_rpc_requests_total'] || 0,
          sync_progress: metrics['aruna_sync_progress'] || 1.0,
          cpu_usage_percent: metrics['aruna_cpu_usage_percent'] || 0.0
        }
      };

      return new Response(JSON.stringify(telemetry), {
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
        status: 200,
      });

    } catch (e) {
      return new Response(JSON.stringify({ error: 'Failed to aggregate telemetry data', details: e.message }), {
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
        status: 502,
      });
    }
  }
};
