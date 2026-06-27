# ADR-0011: P2P Networking Strategy (Custom TCP & libp2p Migration)

## Status
Accepted

## Context
A public blockchain operates on a peer-to-peer (P2P) network. Nodes must be able to discover each other, propagate transaction payloads, broadcast new blocks, synchronize historical headers, and handle network partitions without relying on a centralized coordinator.

The network must function reliably over residential internet connections, home labs, and mobile cellular data networks, which frequently experience packet loss, IP address changes, and firewalls (NAT).

## Problem
Designing and implementing a custom P2P networking stack is a massive engineering effort. Historically, custom stacks suffer from critical vulnerabilities:
1. **Firewall Traversal (NAT):** Nodes in home labs or behind mobile carrier networks cannot accept incoming connections, preventing them from participating in peer routing.
2. **Security Weaknesses:** Custom encryption protocols often introduce handshake vulnerabilities or replay loopholes.
3. **Bandwidth Inefficiency:** Naive gossip protocols waste significant bandwidth by transmitting duplicate blocks and transactions, which can exceed the data limits of mobile and home lab users.
4. **Maintenance Overhead:** Building peer discovery, DHTs, and connection multiplexers from scratch violates the founder protection rule.

We need a phased roadmap that allows building a simple functional testnet prototype quickly while establishing a clear migration path to a robust, standardized stack like libp2p.

## Decision
We adopt a two-phase networking roadmap for the ARUNA Network P2P layer:

### Phase 1: Functional Testnet Prototype (Sumatera Testnet)
* **Custom TCP Protocol:** Build a lightweight, custom TCP protocol using tokio.
* **Length-Prefixed Framing:** Send messages framed with a 4-byte big-endian length prefix followed by serialized Bincode envelopes.
* **Mempool Transaction Gossip:** Relays verified transaction envelopes in a loop-free gossip scheme.
* **Block Synchronization & Gossip:** Connects peers directly and exchanges block structures upon production.

### Phase 2: Public Integration & Scaling (Kalimantan Testnet onwards)
* **rust-libp2p Migration:** Completely migrate the networking stack to standard `rust-libp2p`.
* **Transport Protocols:** Support TCP and QUIC.
* **Kademlia DHT:** Utilize Kademlia for decentralized peer lookup and peer discovery.
* **Gossipsub Protocol:** Replace custom gossip relays with libp2p Gossipsub to optimize traffic and defend against Eclipse/Sybil attacks.

## Alternatives
* **Alternative A: Custom protocol only:** Rejected because a custom stack lacks DHT routing and firewall traversal out-of-the-box, violating long-term scalability and decentralization requirements.
* **Alternative B: Immediate libp2p integration:** Rejected for Phase 1 because libp2p's complex async behavior and configuration boilerplate would delay the functional testnet prototype timeline.

## Consequences
* **Positive:**
  * Fast testnet prototyping using simple, auditable custom TCP.
  * Clear architectural boundaries in `aruna-networking` that keep messages isolated from transport details, simplifying the Phase 2 migration.
* **Negative:**
  * Custom TCP does not support NAT traversal, meaning Phase 1 nodes require public IPs or forwarded ports to accept inbound connections.

## Migration
When transitioning from Phase 1 to Phase 2, `crates/networking`'s transport logic will be replaced with the libp2p behavior while preserving the consensus message types (`P2PMessage`).

## Security Impact
Phase 1 relies on private network setups, firewalls, and manual peer connections. Node identity verification via Node ID checks is enforced during handshakes. Phase 2 will introduce TLS/Noise handshakes and DHT-based defense against Eclipse and DDoS attacks.
