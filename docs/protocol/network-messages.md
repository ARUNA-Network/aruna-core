# Protocol Specification: Network & P2P Messages

This document defines the structures and network byte layouts of all messages transmitted across the libp2p peer-to-peer layer.

## 1. P2P Transport Topologies
ARUNA utilizes **libp2p** for network routing, using two message transmission paradigms:
* **Gossipsub (Publish/Subscribe):** Used for broad, network-wide broadcasts (propagation of new transactions and mined blocks).
* **Request/Response (Direct streams):** Used for point-to-point requests (syncing block history between two nodes).

---

## 2. Handshake Message (`HandshakeMessage`)
When a libp2p connection is established, nodes must execute a handshake to exchange capabilities and verify network alignment before joining gossip groups.

### Handshake Structure:
* **Protocol Version (4 bytes / `u32`):** Current version of the node software.
* **Node ID (32 bytes / `[u8; 32]`):** The BLAKE3 hash of the node's public key.
* **Chain ID (4 bytes / `u32`):** Network identifier (e.g., `1` for Sumatera, `7777` for Jawa Mainnet).
* **Current Height (8 bytes / `u64`):** The height of the canonical tip of the sending node.
* **Capabilities (1 byte / `u8`):** Bit flags representing node capabilities:
  * Bit 0: `FULL_NODE`
  * Bit 1: `VALIDATOR`
  * Bit 2: `ARCHIVE`
  * Bit 3: `RPC`
  * Bit 4: `LIGHT_MINER`

---

## 3. Transaction Propagation Message (`TxMessage`)
Broadcasted over the transaction gossip channel (`/aruna/tx/1`).
* **Fields:**
  * **Serialized Envelope:** The complete Bincode bytes representing the `TransactionEnvelope` (Payload + Signature type + Signature bytes).

---

## 4. Block Propagation Message (`BlockMessage`)
Broadcasted over the block gossip channel (`/aruna/block/1`).
* **Fields:**
  * **Serialized Block Header:** 80-byte header payload.
  * **Serialized Block Body:** Vector of serialized transaction envelopes.
  * **Validator Signature Set:** Merkle proofs of validating signatures.

---

## 5. Synchronization Messages
Direct request-response streams are opened under protocol `/aruna/sync/1`.

### A. Sync Request (`SyncRequestMessage`)
* **Start Height (8 bytes / `u64`):** The height from which to start syncing blocks.
* **End Height (8 bytes / `u64`):** The maximum block height to return.
* **Block Limit (2 bytes / `u16`):** Maximum number of blocks to return in a single response (cap is **500 blocks** to prevent memory exhaustion).

### B. Sync Response (`SyncResponseMessage`)
* **Response Status (1 byte / `u8`):** `0` for Success, `1` for Height Out of Range, `2` for Internal Error.
* **Blocks Vector:** A length-prefixed list of serialized blocks.
