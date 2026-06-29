# ARUNA Explorer API (Cloudflare Worker)

## Overview
Stateless API worker serving block explorer queries from the indexed database.

## Architecture
- **Stateless Edge Execution**: Runs on Cloudflare Edge nodes.
- **Database Access**: Connects to the explorer PostgreSQL instance using Cloudflare Hyperdrive for connection pooling and caching.
- **Cache Policy**: Implements aggressive CDN cache-control headers for static block queries (e.g., historical blocks and finalized transactions).
