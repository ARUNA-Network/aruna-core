# ARUNA Dashboard API (Cloudflare Worker)

## Overview
Stateless Edge worker serving real-time node statistics, telemetry, and system metrics.

## Architecture
- **Metrics Fetching**: Connects securely to the node's `/metrics` and `/status` endpoints.
- **Monitoring Integration**: Aggregates CPU, memory, and P2P peer statistics for display in `apps/dashboard`.
