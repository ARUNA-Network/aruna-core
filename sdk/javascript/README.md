# ARUNA SDK for JavaScript / TypeScript

## Overview
JavaScript client SDK for integrating browser, Node.js, and Cloudflare Worker applications with the ARUNA Network.

## Architecture
- **Local Signing**: Implements client-side transaction serialization and signing (supports Ed25519 and secp256k1).
- **Gateway Communication**: Connects exclusively via `rpc-gateway` HTTPS endpoints.
