# ADR-0011: libp2p Networking

## Status
Proposed

## Context
A public blockchain operates on a peer-to-peer (P2P) network. Nodes must be able to discover each other, propagate transaction payloads, broadcast new blocks, synchronize historical headers, and handle network partitions without relying on a centralized coordinator.

The network must function reliably over residential internet connections, home labs, and mobile cellular data networks, which frequently experience packet loss, IP address changes, and firewalls (NAT).

## Problem
Designing and implementing a custom P2P networking stack is a massive engineering effort. Historically, custom stacks suffer from critical vulnerabilities:
1. **Firewall Traversal (NAT):** Nodes in home labs or behind mobile carrier networks cannot accept incoming connections, preventing them from participating in peer routing.
2. **Security Weaknesses:** Custom encryption protocols often introduce handshake vulnerabilities or replay loopholes.
3. **Bandwidth Inefficiency:** Naive gossip protocols waste significant bandwidth by transmitting duplicate blocks and transactions, which can exceed the data limits of mobile and home lab users.
4. **Maintenance Overhead:** Building peer discovery, DHTs, and connection multiplexers from scratch violates the founder protection rule.

We need a proven, secure, and modular P2P library with built-in NAT traversal, efficient routing, and active maintenance.

## Decision
We select **libp2p** as the official networking protocol library for ARUNA Network (`crates/networking`).

### Core libp2p Configuration & Features:
1. **Transport Protocols:**
   * **TCP:** The primary transport layer for stable node-to-node streaming.
   * **QUIC (Future):** Integrated for faster handshakes and better connection stability on mobile carriers.
2. **Peer Discovery Protocols:**
   * **Bootstrap Discovery:** Nodes obtain initial peer lists from hardcoded bootstrap servers (e.g., `bootstrap.aruna.network`).
   * **Kademlia DHT:** Used for decentralized peer lookup, peer routing, and advertising node capabilities.
   * **PEX (Peer Exchange):** Active peers share lists of known connected nodes to bypass DHT latency.
3. **Gossipsub Protocol:**
   * Used for transaction, block, and governance message propagation. Gossipsub provides structured gossip limits to minimize redundant messages, reducing bandwidth overhead for mobile miners.
4. **NAT Traversal:**
   * Utilizes UPnP, STUN, and TURN protocols to enable direct peer connections for nodes operating behind firewalls and NATs.
5. **Connection Limits (Eclipse Attack Defense):**
   * Nodes must maintain a **minimum of 16 connections** (recommended 32+) with geographically diverse peers to prevent routing isolation (Eclipse attacks).

## Alternatives
* **Alternative A: Custom TCP/UDP Socket Protocol:** Similar to early Bitcoin implementations. This was rejected because the engineering effort required to build secure encryption, DHT discovery, and robust NAT traversal is too high for a small team.
* **Alternative B: Devp2p (Ethereum's stack):** Highly specialized for EVM. However, Rust support is less mature compared to the modular `rust-libp2p` ecosystem, which has wide adoption and excellent documentation.

## Consequences
* **Positive:**
  * **Zero-Cost Security:** Uses noise handshakes and TLS encryption out-of-the-box.
  * **Interoperability:** Modular architecture allows swap-in transports (e.g. switching from TCP to QUIC) without rewriting message validation logic.
  * **Mobile Viability:** libp2p's light protocols are efficient, saving battery and data on Android nodes.
* **Negative:**
  * libp2p introduces a complex asynchronous codebase, which can result in steep compilation times and detailed configuration tuning for Tokio runtime integration.

## Migration
Not applicable. The libp2p engine is the native network coordinator from launch.

## Security Impact
libp2p mitigates P2P routing exploits. Handshakes validate peer IDs (`BLAKE3(Node PublicKey)`), preventing node impersonation. The Gossipsub mechanism rejects invalid block/transaction sizes before relaying, defending the network against P2P DDoS floods.
