# ARUNA Cloudflare Integration

## Overview
Edge caching, routing, and deployment specifications for the ARUNA Network platform.

## Architecture
- **Cloudflare Pages**: Hosts the static web applications (`apps/*`).
- **Cloudflare Workers**: Executes stateless proxy and API logic (`workers/*`).
- **Cloudflare Hyperdrive**: Connection pooling and read caching for PostgreSQL.
- **Cloudflare Tunnel**: Secure localhost forwarding to raw Node RPC ports.
