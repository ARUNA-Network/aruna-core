# ARUNA Web Wallet (Static Frontend)

## Overview
A secure, browser-based wallet client for the ARUNA Network blockchain.

## Platform Boundaries
- **No Private Keys on Server**: All transaction signing must occur locally using browser-secure storage (IndexedDB with WebCrypto API) or integration with external hardware wallets.
- **RPC Communication**: Connects exclusively to the ARUNA Edge Gateway (`workers/rpc-gateway`), never directly to raw node ports.
- **Stateless Structure**: This is a static application, easily hostable on Cloudflare Pages or general CDNs.
