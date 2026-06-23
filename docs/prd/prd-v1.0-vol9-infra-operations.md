# ARUNA NETWORK

## MASTER PRD

### Volume 9 — Infrastructure, Deployment & Operations

---

# INFRASTRUCTURE PHILOSOPHY

ARUNA infrastructure is designed to be:
* Low Cost
* Open Source
* Self Hostable
* Community Operable
* Cloud Independent
* Fault Tolerant

*The network must survive even if founder-operated infrastructure goes offline.*

---

# INFRASTRUCTURE TIERS

* **Tier 1 (Community Devices):** Android, Raspberry Pi, Mini PC (Mining, Wallet, Light Node).
* **Tier 2 (Home Labs):** NUC, Mini Server, Xeon Server, ARM Server (Full Node, RPC, Validator).
* **Tier 3 (Community Infrastructure):** Seed Nodes, Archive Nodes, Explorer Nodes.
* **Tier 4 (Public Infrastructure):** Explorer, Website, Bootstrap Services, Public RPC.

---

# HOMELAB REFERENCE ARCHITECTURE (FOUNDER)

```
Homelab Server
├── aruna-node
├── aruna-validator
├── aruna-rpc
├── aruna-indexer
├── postgres (database)
├── prometheus (metrics)
├── grafana (visualization)
└── backup-agent
```

---

# DEPLOYMENT MODEL

* **Model:** Container First.
* **Standard:** Docker and Docker Compose (Future: Kubernetes, not MVP).
* **Node Deployment:** Nodes and auxiliary tools packaged as Docker images (`aruna/node`, `aruna/indexer`, `aruna/rpc`, `aruna/explorer`).

---

# SEED & VALIDATOR INFRASTRUCTURE

### Seed Node Architecture:
* **Purpose:** Peer discovery (bootstrap).
* **Requirements:** Stable IP, 24/7 availability, high uptime.
* **Initial Mainnet Setup:** Minimum 5 seed nodes distributed across Indonesia, Singapore, Japan, Germany, and the United States.

### Validator Infrastructure Specs:
* **Minimum Specs:** 4 CPU, 8 GB RAM, 100 GB SSD.
* **Recommended Specs:** 8 CPU, 16 GB RAM, 500 GB NVMe SSD.
* **Requirements:** 24/7 uptime, reliable internet, automatic crash recovery.

---

# RPC & INDEXER INFRASTRUCTURE

* **Isolation Rule:** **RPC Node ≠ Validator Node.** Never expose validator nodes directly. Public and private RPC queries must route to dedicated RPC instances.
* **Indexer Sync Flow:**
```
Node → Block Stream → Indexer → PostgreSQL → Explorer
```

---

# CLOUDFLARE & DOMAIN STRATEGY

* **Cloudflare:** Used for DNS, CDN, SSL, Pages, DDoS protection, and caching.
* **Rule:** **Do NOT store consensus state in Cloudflare.** All consensus nodes remain self-hosted.
* **Domains:** Phase 1 uses `arunachain.org` (or cheapest suitable domain). Phase 2 targets `aruna.network`.

---

# CI/CD STRATEGY

* **Platform:** GitHub Actions.
* **PR Policy:** Every Pull Request must compile, test, lint, and undergo security scans.

---

# OBSERVABILITY & LOGGING

* **Metrics Stack:** Prometheus & Grafana.
* **Metrics Tracked:** Block height, hashrate, active validators, peers, txs, CPU/RAM/Disk usage.
* **Logging:** Rust `tracing` with standard log levels (ERROR, WARN, INFO, DEBUG, TRACE).

---

# BACKUP & DISASTER RECOVERY

* **Backup Rule (3-2-1 Strategy):** Keep 3 copies of critical assets (Node config, Validator keys, Postgres databases, State snapshots), using 2 different media types, with at least 1 offsite copy.
* **Disaster Recovery Goals:**
  * Node recovery: < 1 Hour.
  * RPC recovery: < 30 Minutes.
  * Explorer recovery: < 2 Hours.

---

# RELEASE & STAGING TIMELINE

* **Testnet Sumatera (Year 1):** Protocol validation. Requires minimum 2 Seed nodes, 5 Full nodes, and 20+ community testers.
* **Testnet Kalimantan (Year 2):** Stress testing.
* **Testnet Sulawesi (Year 3):** Economic testing.
* **Papua RC (Year 4):** Mainnet rehearsal.
* **Mainnet Readiness Checklist (Year 5):** Requires stable consensus/wallet/explorer/mining, complete audits, and minimum launch targets of **100+ nodes, 20+ validators, and 1000+ wallets** before Jawa Mainnet launch.

---

# INCIDENT RESPONSE & COST MANAGEMENT

* **Incident Severity:** L1 (UI) → L2 (Node) → L3 (Network) → L4 (Consensus) → L5 (Chain Threat).
* **Cost Optimization Goal:**
  * Phase 1 (Founder Homelab): ≈ Rp 0 – Rp 300,000 / month.
  * Phase 2 (Community Nodes): Shared infrastructure costs.
  * Phase 3: Complete community-operated decentralized infrastructure.

---

# SUCCESS CRITERIA

Infrastructure succeeds when:
1. The founder can operate bootstrap nodes alone.
2. Staking and indexer configurations are reproducible by the community.
3. Node hosting remains cheap and cloud-independent.
4. The blockchain survives the loss of the founder's seed nodes.

End of Volume 9.
