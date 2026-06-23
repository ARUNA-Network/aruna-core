# ADR-0012: Node Types

## Status
Proposed

## Context
A public blockchain network is sustained by different participants with varying hardware capabilities, bandwidth limits, and operational objectives. An Android smartphone has different storage and CPU capacity compared to a home lab server or a cloud server virtual machine.

To enable fair participation, the network must support different node types, allowing users to choose the role that matches their device constraints.

## Problem
Treating all network nodes identically creates a centralization bottleneck:
1. **Storage Exhaustion:** If every node must act as an archive node (storing the entire historical state and transaction history), consumer hardware (Mini PCs, Raspberry Pi, Android) will rapidly run out of disk space, centralizing the network onto cloud storage providers.
2. **CPU Overload:** Forcing mobile devices to validate and execute every historical contract transition will cause them to overheat and drain their batteries.
3. **Bandwidth Saturation:** Light miners only require block templates, not the full transaction payload of every block.

We need a structured node classification model that defines specific responsibilities and hardware specifications for each network role.

## Decision
We define the following **ARUNA Node Types** in our architecture (`crates/node`):

| Node Type | Primary Purpose | Chain Data | Consensus Role | Hardware Requirements |
| :--- | :--- | :--- | :--- | :--- |
| **Seed Node** | Peer discovery & network bootstrapping | None (Headers only) | None | Stable IP, 24/7 uptime, low storage, low RAM |
| **Full Node** | Full state verification & relaying | Full block history + State | Relays blocks/txs | 4 CPU, 8 GB RAM, 100 GB SSD |
| **Validator Node** | Block validation & signing | Full block history + State | Active voting & PoS | 8 CPU, 16 GB RAM, 500 GB NVMe |
| **Archive Node** | Historical storage & explorer indexing | Complete history (non-pruned) | Serve historical queries | 16 CPU, 32 GB RAM, 2 TB+ SSD |
| **RPC Node** | Application access and wallet queries | Full block history + State | Serves APIs | 8 CPU, 16 GB RAM, 250 GB SSD |
| **Indexer Node** | Explorer search & analytics indexing | Synced Postgres Database | Extracts and catalogs state | 4 CPU, 8 GB RAM, 500 GB SSD |
| **Light Node (Mobile)**| Wallet, voting, and mining | Headers + Checkpoints | Validates recent state | Android Phone, Raspberry Pi, 2 GB RAM |

### Key Node Guidelines:
1. **Capabilities Exchange:** During the libp2p handshake, nodes must broadcast their type and capabilities. This allows light nodes to connect specifically to full/RPC nodes to download block metadata.
2. **Validator Isolation:** Validator nodes should operate on private subnets. They must connect only to dedicated sentry full nodes, never directly exposing their interface to public RPC or web traffic.

## Alternatives
* **Alternative A: Monolithic Node (All nodes are Full Nodes):** Rejected because it makes mobile wallets and screen-off mobile mining impossible, as phones cannot host the gigabytes of blockchain state.
* **Alternative B: Pure Client-Server Model (No mobile nodes):** Mobile wallets connect to a centralized API provider (like Infura). This was rejected because it reduces user privacy and increases centralized dependency.

## Consequences
* **Positive:**
  * **Device Inclusivity:** Anyone can participate; an Android phone can run as a Light Node miner, while a home lab server operates as a Validator Node.
  * **Ecosystem Modularity:** Explorer databases (Indexer Node) are decoupled from the consensus state (Full Node), preventing indexer crashes from bringing down validators.
* **Negative:**
  * Multi-node architectures require complex peer routing and data synchronization logic in the P2P networking layer.

## Migration
Not applicable. The node capabilities structure is established at project initialization.

## Security Impact
Node type classification prevents resource depletion attacks. RPC rate limits protect RPC nodes, while sentry nodes protect validator signatures. This ensures that validator nodes remain isolated, preventing DDoS attacks from blocking consensus participation.
