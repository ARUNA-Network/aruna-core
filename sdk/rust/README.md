# ARUNA SDK for Rust

## Overview
Rust client SDK for building high-performance services, miners, and tools interacting with the ARUNA Network.

## Architecture
- **Wrapper API**: Wraps core primitives (`aruna-primitives`) and networking APIs.
- **RPC Client**: Communicates via async HTTP/WSS requests through `rpc-gateway`.
