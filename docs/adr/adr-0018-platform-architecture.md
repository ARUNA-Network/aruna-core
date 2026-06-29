# ADR-0018: Platform Architecture Evolution

## Context
As the ARUNA Network evolves from a standalone core blockchain daemon into a complete decentralized platform ecosystem, we must establish clear architectural boundaries and folder contracts. This prevents dependency pollution and ensures structural scalability.

## Decision
We decouple platform components into three main structural directories:
1. **`apps/`**: Operator and end-user visual applications (static frontends only).
2. **`workers/`**: Cloudflare Edge microservices implementing stateless business logic (caching, rate-limiting, search orchestration).
3. **`sdk/`**: Unified API wrappers for developers and clients.

```
                  ┌──────────────────────┐
                  │    User Browser /    │
                  │     Mobile App       │
                  └──────────┬───────────┘
                             │ HTTPS / WSS
                             ▼
                  ┌──────────────────────┐
                  │ Cloudflare Edge      │
                  │ (workers/rpc-gateway)│
                  └──────────┬───────────┘
                             │ Forwarded RPC
                             ▼
                  ┌──────────────────────┐
                  │ ARUNA Node           │
                  │ (crates/node/)       │
                  └──────────────────────┘
```

## Platform Contracts
- **Core Decoupling**: The core Rust workspace (`crates/*`) must never import or depend on anything inside `apps/`, `workers/`, or `sdk/`.
- **Stateless Workers**: All edge services in `workers/` must remain completely stateless. Any persistent state must go through databases (e.g., PostgreSQL via Hyperdrive) or the blockchain node.
- **Client Access**: Client apps (`apps/wallet-web`, `apps/explorer-ui`) should ideally interface with Edge Workers (`workers/*`) rather than talking directly to raw TCP/RPC ports of the blockchain node.
