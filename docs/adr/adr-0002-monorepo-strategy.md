# ADR-0002: Monorepo Strategy

## Status
Proposed

## Context
ARUNA Network consists of a complete software ecosystem, including the core node (`aruna-node`), cryptography and hashing engines (`crates/crypto`, `crates/ahash`), networking libraries (`crates/networking`), indexer services (`crates/indexer`), block explorers (`apps/explorer`), wallets (`apps/wallet-mobile`, `apps/wallet-desktop`), and command-line interfaces.

Managing these components across multiple Git repositories creates substantial overhead. In a decentralized project run by a solo founder with the assistance of autonomous AI agents, keeping code, configuration, dependency maps, and documentation synchronized is critical.

## Problem
Using a multi-repository strategy introduces the following challenges for a solo founder and AI-assisted workflow:
1. **Dependency Desynchronization:** A change in the core primitives or consensus crate requires updating and releasing multiple individual repositories, leading to version mismatch and compilation errors.
2. **AI Context Constraints:** AI agents need a single source of truth to reason about cross-crate interactions. Multi-repo setups make it difficult for agents to locate and analyze code dependencies concurrently.
3. **Commit Atomicity:** Features that span both protocol crates and client applications (such as wallet transaction signing) cannot be checked in, reviewed, or tested in a single atomic commit.
4. **CI/CD Overhead:** Operating multiple CI/CD pipelines (GitHub Actions) increases maintenance complexity and cost.

We need a repository organization model that minimizes integration friction and supports clean, AI-friendly workspace navigation.

## Decision
We adopt a **Single Monorepo** strategy under the repository name `aruna-core` (or `aruna-network`), organized as a Cargo and Node workspace. The root repository structure is defined as:

```
/
├── apps/               # User-facing applications (Nuxt, Flutter)
│   ├── explorer/       # Nuxt block explorer
│   ├── wallet-mobile/  # Flutter mobile wallet
│   ├── wallet-desktop/ # Flutter desktop wallet
│   ├── miner-mobile/   # Flutter mining dashboard (Android)
│   └── miner-desktop/  # Flutter desktop mining dashboard
├── crates/             # Core protocol Rust libraries
│   ├── primitives/     # Base types (Hash, Address, Block, Tx)
│   ├── crypto/         # Cryptographic primitives (BLAKE3, Ed25519)
│   ├── ahash/          # Custom AHash mining algorithm
│   ├── consensus/      # PoW, PoS, and Treasury consensus engine
│   ├── execution/      # Deterministic state transition engine
│   ├── state/          # Account state storage (RocksDB)
│   ├── storage/        # Block persistence and snapshots
│   ├── networking/     # P2P libp2p implementation
│   ├── mempool/        # Transaction queue and verification
│   └── node/           # Composition layer and startup logic
├── docs/               # Architecture documents and ADRs
│   ├── prd/            # Master Product Requirements Documents
│   └── adr/            # Architecture Decision Records
└── infrastructure/     # Docker Compose, Prometheus, Grafana configs
```

### Key Repository Guidelines:
1. **Primitives Rule:** All protocol crates must depend on `crates/primitives`. `crates/primitives` depends on nothing.
2. **Modular Monolith First:** Crates must remain highly decoupled with clear API boundaries. Decoupling is maintained so components can be separated into distinct repositories in future phases (e.g., Year 5+) without rewriting codebase logic.

## Alternatives
* **Alternative A: Multi-Repository Strategy:** Separate repositories for consensus, wallet, miner, and explorer. This was rejected because the maintenance overhead of managing Git submodules, packages, and separate PRs is too complex for a solo developer.
* **Alternative B: Monolith (Single Crate):** Build all core logic into a single monolithic Rust crate. This was rejected because it violates the replaceability rule: a developer must be able to swap out `crates/networking` or `crates/state` without touching consensus or transaction structures.

## Consequences
* **Positive:**
  * **Atomicity:** Cross-crate changes can be pushed, built, and tested in a single commit.
  * **AI Compatibility:** Agents can read, understand, and modify the entire codebase with complete context.
  * **Unified Tooling:** Easier linting, formatting, testing, and CI/CD setup via GitHub Actions.
* **Negative:**
  * Git clone sizes will increase as the wallet and explorer codebases grow alongside the node.
  * Dependency graph changes in the monorepo require careful Cargo caching in CI/CD to maintain fast build times.

## Migration
Existing documents and configuration files are consolidated under the root workspace folders `docs/` and `.agents/`.

## Security Impact
A single monorepo allows CI/CD to run comprehensive integration tests and security scans across all modules on every pull request. However, it also means a compromise of the main repository repository credentials grants access to both core consensus logic and client application wallets. Branch protection rules and signed commits must be strictly enforced on `main`.
