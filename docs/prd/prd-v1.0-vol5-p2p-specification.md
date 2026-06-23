# ARUNA NETWORK

## MASTER PRD

### Volume 5 — Network Protocol & P2P Specification

---

# NETWORK PHILOSOPHY

ARUNA networking is designed for:
* Consumer Internet
* Mobile Networks
* Home Labs
* ARM Devices
* Intermittent Connectivity

### Resiliency Assumptions:
* Peers disappear frequently.
* IP addresses change.
* Mobile devices sleep.
* Connections fail.
* **Network resiliency is mandatory.**

---

# NETWORK LAYERS

```
Application Layer
        │
        ▼
RPC Layer
        │
        ▼
P2P Layer
        │
        ▼
Transport Layer
        │
        ▼
TCP / QUIC
```

---

# NODE IDENTITY

Each node generates a **Node Private Key** and **Node Public Key** upon first startup.
* **NodeID Formula:** `NodeID = BLAKE3(PublicKey)`
* **Properties:** Globally unique, persistent, self-generated, no central registration.

---

# PEER DISCOVERY

* **Library:** libp2p.
* **Bootstrap Discovery:** Seed connections obtained initially via `bootstrap.aruna.network`.
* **Seed Discovery:** Stable known peer list and metadata anchors.
* **DHT Discovery:** Distributed Hash Table for peer lookup, peer advertisement, and peer exchange (PEX).

---

# BOOTSTRAP NODES

* **Purpose:** First network entry point.
* **Responsibilities:** Peer discovery, initial routing, network metadata.
* *Not responsible for consensus, validation, or governance.*

---

# SEED NODES

* **Purpose:** Provide stable network anchors.
* **Recommended Deployments:** Indonesia, Singapore, Japan, Germany, United States.
* **Initial Mainnet Setup:** Minimum of 5 Seed Nodes owned by independent operators.

---

# HANDSHAKE PROTOCOL

Peers must exchange the following metadata:
1. **Protocol Version**
2. **Node ID**
3. **Chain ID**
4. **Current Height**
5. **Capabilities** (e.g., `FULL_NODE`, `VALIDATOR`, `ARCHIVE_NODE`, `RPC_NODE`, `MINER`)
6. **Connection Validation & Acceptance**

---

# PEER REPUTATION SYSTEM

* **Base Score:** Starts at 100.
* **Positive Events:** Valid blocks, valid transactions, and stable uptime increase score.
* **Negative Events:** Spam, invalid blocks/transactions, and protocol violations decrease score.
* **Action:** Low reputation peers are throttled or disconnected/banned.

---

# GOSSIP PROTOCOL

* **Used For:** Transactions, blocks, votes, and governance messages.
* **Mechanism:** Receive → Validate → Forward (invalid messages are dropped).

---

# PROPAGATION FLOWS

### Transaction Propagation:
```
Wallet → Node → Mempool → Peer Network → Validators → Block
```
* *Validate signature, nonce, balance, and fee before propagation.*

### Block Propagation:
```
Miner → Block Found → Broadcast → Peers → Entire Network
```
* *Perform full validation before relaying blocks. Reject invalid blocks immediately.*

---

# MEMPOOL & CHAIN SYNCHRONIZATION

* **Mempool Sync:** Inventory Announcements (similar to Bitcoin) to reduce bandwidth and avoid duplicate transfers.
* **Chain Sync:** Startup nodes connect, discover peers, sync headers first, sync blocks, and validate the chain.
* **Fast Sync:** Light Nodes download Headers First, perform Checkpoint Validation, and download the Recent State (avoiding the entire historical chain).
* **Snapshot Sync:** Verified State Snapshot download for fast onboarding.

---

# MOBILE NODE STRATEGY

Android nodes are classified as **Light Nodes** by default:
* Low bandwidth, low storage, low memory usage.
* Can act as wallets, participate in mining, governance, and staking.
* *Not required to store the full block history.*

---

# ANDROID MINING PROTOCOL

* **Client:** Miner-Light.
* **Workflow:** Sync Header → Receive Work Template → Run AHash → Submit Share.
* **Thermal Management:** Monitor CPU & Battery temperatures. Reduce hashrate if thresholds are exceeded. Pause mining at critical thresholds.
* **Battery Protection:** Stop mining if battery drops below 20% (unless user overrides). Resume mining when charging.
* **Screen Off:** Mining must function reliably with screen off.

---

# ATTACK RESISTANCE & DEFENSES

* **Anti-Sybil Strategy:** Multi-layered protection using:
  * Economic Cost (PoW mining & PoS staking costs).
  * Reputation System (persistent score tracking).
  * Stake Weight (validator participation needs stake).
  * Peer Diversity (prefer geographically diverse peers).
* **Eclipse Attack Defense:** Nodes must maintain multiple independent peers (minimum 16 connections, 32+ recommended).
* **DDoS Resistance:** Rate limiting, connection limits, message validation, peer reputation, and adaptive temporary or long-term bans.
* **Bandwidth Optimization:** Header-First Sync, compression, dropping duplicate messages, and inventory systems.

---

# SUCCESS CRITERIA

The network layer succeeds when:
1. Android devices participate reliably.
2. ARM devices sync efficiently.
3. Home labs remain viable.
4. Mobile networks remain usable.
5. DDoS and Sybil attacks are mitigated.
6. Eclipse attacks are prevented.
7. Nodes recover quickly from disconnections.

End of Volume 5.
