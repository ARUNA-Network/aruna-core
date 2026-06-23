# ARUNA NETWORK

## MASTER PRD

### Volume 4 — Core Technical Architecture

---

# ARCHITECTURE PHILOSOPHY

ARUNA follows:
**Modular Monolith First, Distributed Architecture Later**

### Reason:
A solo developer should not build a microservice ecosystem prematurely.
The protocol should initially remain:
* Modular
* Testable
* Replaceable
*without becoming operationally complex.*

---

# CORE DESIGN PRINCIPLES

Every subsystem must be:
* Independent
* Deterministic
* Replaceable
* Testable
* Observable
* Documented

---

# HIGH LEVEL ARCHITECTURE

ARUNA Core
├── Protocol Layer
├── Consensus Layer
├── Execution Layer
├── State Layer
├── Storage Layer
├── Networking Layer
├── RPC Layer
├── Indexing Layer
└── Governance Layer

---

# MONOREPO STRATEGY

Single Repository (Monorepo)
* **Reason:** Easier management, better AI context, easier protocol evolution.
* **Repository Name:** `aruna-network`

---

# ROOT REPOSITORY STRUCTURE

```
/
├── apps
├── crates
├── docs
├── scripts
├── infrastructure
├── tests
├── tools
└── .github
```

---

# APPS

User-facing software:
* `apps/explorer` (Nuxt frontend)
* `apps/wallet-mobile` (Flutter)
* `apps/wallet-desktop` (Flutter)
* `apps/miner-mobile` (Flutter)
* `apps/miner-desktop` (Flutter)
* `apps/dashboard`

---

# CRATES

Core protocol libraries under `crates/`:
* `crates/ahash` (Mining implementation)
* `crates/consensus` (PoW, PoS, and Treasury logic)
* `crates/execution` (Deterministic state transitions)
* `crates/state` (RocksDB account & contract state)
* `crates/storage` (Data persistence, archive, snapshots)
* `crates/networking` (libp2p implementations)
* `crates/mempool` (Transaction queue and validation)
* `crates/transaction` (Structures, validations, fee calculation)
* `crates/validator` (Registry and consensus details)
* `crates/staking` (Stake and rewards accounting)
* `crates/governance` (Proposal and voting logic)
* `crates/treasury` (Protocol-level reserve accounts)
* `crates/rpc` (HTTP & WebSocket access layer)
* `crates/indexer` (PostgreSQL sync pipeline)
* `crates/crypto` (Key generation, signatures, hashes)
* `crates/primitives` (Base types like Hash, Address, Block)
* `crates/node` (Composition layer initializing modules)

---

# PRIMITIVES CRATE

Contains base structures:
* Hash, Address, Block, Transaction, Signature, Nonce, Difficulty, ChainId.

* **Dependency Rule:** All protocol modules depend on `primitives`. `primitives` depends on nothing.

---

# CRYPTO CRATE

* **Responsibilities:** Hashing, signatures, key generation, verification, and cryptographic utilities.
* **Supported Algorithms:** Ed25519, secp256k1, BLAKE3, AES, Argon2.

---

# AHASH CRATE

* **Responsibilities:** Native mining algorithm, difficulty validation, hash verification, benchmarking, hardware/architecture optimization.
* **Target Platforms:** Android, ARM64, x86_64.

---

# TRANSACTION CRATE

* **Responsibilities:** Structure, fee calculation, validation rules, and serialization.
* **Types:** Transfer, Stake, Unstake, Vote, Deploy Contract, Execute Contract, DEX Transaction.

---

# CONSENSUS CRATE

* **Responsibilities:** Block validation, difficulty rules, fork choice, reward/treasury distribution.
* Contains PoW, PoS, and Treasury logic.

---

# STAKING CRATE

* **Responsibilities:** Validator registry, delegation tracking, reward calculation, and stake accounting.

---

# GOVERNANCE CRATE

* **Responsibilities:** Proposal system, voting logic, upgrade voting, and treasury spending controls.

---

# TREASURY CRATE

* **Responsibilities:** Treasury accounting, state tracking, and protocol governance integration.

---

# EXECUTION ENGINE

* **Responsibilities:** Deterministic transaction execution, contract execution, fee/reward processing, and state updates.
* **Rule:** Must be strictly deterministic. No nondeterministic behavior allowed.

---

# STATE ENGINE

* **Responsibilities:** Balances, validator state, governance state, treasury state, and contract state.
* **Storage Backend:** RocksDB.

---

# STORAGE ENGINE

* **Responsibilities:** Persist blocks/txs, snapshot management, state recovery, and historical queries.
* **Storage Layers:** Hot State, Archive State, Snapshots.

---

# MEMPOOL

* **Rules:** Reject invalid transactions, reject double spend, reject invalid signatures. Prioritize higher fees.

---

# NETWORKING ENGINE

* **Library:** libp2p.
* **Responsibilities:** Peer discovery, block/transaction propagation, synchronization, fork detection.
* **Protocols:** Peer, Transaction, Block, Sync.

---

# NODE CRATE

* **Purpose:** System composition and startup layer.
* **Responsibilities:** Initialize modules, manage lifecycle, service management, configuration loading.

---

# RPC ARCHITECTURE

* **Transport:** HTTP, WebSocket (Future: gRPC).
* **Namespaces:** `chain_`, `block_`, `transaction_`, `wallet_`, `validator_`, `governance_`, `dex_`.

---

# INDEXER ARCHITECTURE

* **Purpose:** Explorer support, wallet search, and analytical support.
* **Database:** PostgreSQL.
* **Flow:** Node → Block Stream → Indexer → PostgreSQL → Explorer.

---

# CONFIGURATION & LOGGING STRATEGY

* **Configuration:** All configurations must be in TOML format (e.g., `config.toml`). No hardcoded ports, peers, or configurations.
* **Logging:** Utilizes the `tracing` library. Required levels: ERROR, WARN, INFO, DEBUG, TRACE.

---

# OBSERVABILITY

* **Metrics:** Prometheus.
* **Dashboards:** Grafana.
* **Monitored Metrics:** Blocks, peers, validators, transactions, memory, and CPU usage.

---

# UPGRADE STRATEGY

* Upgrades require an approved ADR, Security Review, Migration Plan, and Governance Approval.
* **No emergency protocol changes or founder overrides.**

---

# ADR FRAMEWORK

* Architecture Decision Records (ADRs) are stored under `docs/adr/` in format `ADR-XXXX`.
* **Required Sections:** Context, Problem, Decision, Alternatives, Consequences, Migration, Security Impact.
* **Rule:** No protocol modification without an ADR.

---

# TESTING STRATEGY

* **Test types:** Unit, integration, consensus, fork, load, property tests.
* **Code Coverage Goal:** Minimum 80% overall.
* **Consensus Modules Goal:** 95%+ coverage.

---

# AI AGENT COMPATIBILITY

* To enable efficient AI-assisted development, every crate must have:
  * README file
  * ADR references
  * Architecture notes
  * Code examples
  * High-coverage tests

---

# SUCCESS CRITERIA

The architecture succeeds when:
1. A solo developer can maintain it.
2. New contributors can easily understand it.
3. AI agents can autonomously navigate it.
4. Consensus remains deterministic.
5. Modules remain replaceable.
6. The system can evolve for decades.

End of Volume 4.
