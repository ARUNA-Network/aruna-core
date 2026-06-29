# ARUNA RPC Gateway (Cloudflare Worker)

## Overview
Critical Cloudflare Edge service acting as the public-facing gateway for all Node RPC queries.

## Architecture
- **Stateless Proxying**: Forwards incoming RPC requests to internal ARUNA Node pools (via Cloudflare Tunnels).
- **Edge Caching**: Caches idempotent queries (e.g., historical blocks by hash/height) to reduce node RPC overhead.
- **Rate Limiting**: Enforces rate limit floors on public endpoints to mitigate DDoS and spam.
- **Request Validation**: Parses and validates payload formats before routing to the node.
