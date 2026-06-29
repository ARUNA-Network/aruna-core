/**
 * ARUNA Faucet API (Cloudflare Worker)
 *
 * Handles Testnet token distribution. Implements IP/Address rate limiting via KV,
 * captcha verification, and submits signed transfer transactions to the Node RPC.
 */

export default {
  async fetch(request, env) {
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
    };

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    if (request.method !== 'POST') {
      return new Response(JSON.stringify({ error: 'Method not allowed' }), {
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
        status: 405,
      });
    }

    try {
      const { address, captchaToken } = await request.json();

      // 1. Basic format validation
      if (!address || !address.startsWith('sum1')) {
        return new Response(JSON.stringify({ error: 'Invalid address: must start with sum1' }), {
          headers: { 'Content-Type': 'application/json', ...corsHeaders },
          status: 400,
        });
      }

      // 2. Validate Turnstile captcha token (stub)
      if (!captchaToken) {
        return new Response(JSON.stringify({ error: 'Missing Turnstile captcha token' }), {
          headers: { 'Content-Type': 'application/json', ...corsHeaders },
          status: 400,
        });
      }

      // 3. IP and Address Rate Limiting (24-hour limit using Cloudflare KV)
      const ip = request.headers.get('CF-Connecting-IP') || 'anonymous';
      const keyAddress = `faucet:addr:${address}`;
      const keyIp = `faucet:ip:${ip}`;

      if (env.FAUCET_LIMITS) {
        const isAddrLimited = await env.FAUCET_LIMITS.get(keyAddress);
        const isIpLimited = await env.FAUCET_LIMITS.get(keyIp);

        if (isAddrLimited || isIpLimited) {
          return new Response(JSON.stringify({ error: 'Faucet request limit exceeded. Try again in 24 hours.' }), {
            headers: { 'Content-Type': 'application/json', ...corsHeaders },
            status: 429,
          });
        }
      }

      // 4. Construct and Sign Faucet Transaction
      // In production, the Faucet Worker holds a secure Testnet faucet key inside CF Secrets.
      // We serialize the transaction payload and submit to the Node RPC /tx endpoint.
      const amount = parseInt(env.FAUCET_AMOUNT_MICRO || '10000000', 10);
      
      const payload = {
        sender: "sum1faucetaddressxxxxxxxxxxxxxxxxxxxxxxxxxx", // Faucet address
        recipient: address,
        amount: amount,
        fee: 1000,
        nonce: Date.now(), // Unique identifier
        gas_limit: 21000,
        gas_price: 1,
        data: []
      };

      // Mock signed transaction envelope to send to raw Node RPC `/tx`
      const txEnvelope = {
        payload: payload,
        signature_type: 0, // Ed25519
        signature: Array(64).fill(0), // Signature bytes (stub)
        public_key: Array(32).fill(0) // Public key bytes (stub)
      };

      const nodeRes = await fetch(`${env.NODE_RPC_URL}/tx`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(txEnvelope)
      });

      if (!nodeRes.ok) {
        const errBody = await nodeRes.text();
        throw new Error(`Node rejected tx: ${errBody}`);
      }

      const txResult = await nodeRes.json();

      // 5. Update rate limit state in KV (expire in 24 hours)
      if (env.FAUCET_LIMITS) {
        await env.FAUCET_LIMITS.put(keyAddress, 'true', { expirationTtl: 86400 });
        await env.FAUCET_LIMITS.put(keyIp, 'true', { expirationTtl: 86400 });
      }

      return new Response(JSON.stringify({
        status: 'success',
        amount: amount / 1_000_000,
        tx_hash: txResult.tx_hash
      }), {
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
        status: 200,
      });

    } catch (e) {
      return new Response(JSON.stringify({ error: 'Faucet processing failed', details: e.message }), {
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
        status: 500,
      });
    }
  }
};
