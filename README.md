# ARUNA Network Core

[![ARUNA CI](https://github.com/ARUNA-Network/aruna-core/actions/workflows/ci.yml/badge.svg)](https://github.com/ARUNA-Network/aruna-core/actions/workflows/ci.yml)
[![ARUNA Nightly Performance Benchmark](https://github.com/ARUNA-Network/aruna-core/actions/workflows/bench.yml/badge.svg)](https://github.com/ARUNA-Network/aruna-core/actions/workflows/bench.yml)

> *"Dari Rakyat. Oleh Rakyat. Untuk Rakyat."*  
> *"Mine Anywhere. Owned By Everyone."*

ARUNA Network is a community-owned, payment-focused hybrid blockchain originating from Indonesia. Natively optimized for low-power and ARM architectures, ARUNA enables energy-efficient background mining on everyday consumer hardware (Android, ARM64, Raspberry Pi, ARM Servers) alongside traditional x86 systems.

---

## 🚀 Key Specifications & Architecture

- **Account Model**: Account-based architecture (Address, Balance, Nonce, Code, Storage) supporting future EVM compatibility.
- **Consensus Mechanism**: Hybrid Proof of Work (PoW) and Proof of Stake (PoS) consensus combining cumulative work weight and finalized stake weight.
- **Block Parameters**: 30-second target block time, 2 MB size limit, soft finality at 4 blocks (~2 minutes).
- **Difficulty Adjustment**: Weighted Moving Average (WMA) adjusting target difficulty every block.
- **State Storage**: RocksDB backend with deterministic, lexicographically sorted Merkle state root transitions.
- **Address Format**: Bech32m encoding utilizing regional network prefixes:
  - *Sumatera Testnet*: `sum1` (Contract: `sumc1`)
  - *Kalimantan Testnet*: `kal1` (Contract: `kalc1`)
  - *Sulawesi Testnet*: `sul1` (Contract: `sulc1`)
  - *Papua Release Candidate*: `pap1` (Contract: `papc1`)
  - *Jawa Mainnet*: `jaw1` (Contract: `jawc1`)
- **HD Derivation Path**: `m/44'/7777'/0'/0/0` (ARUNA coin type `7777`), using standard BIP-39 seed phrases.

### Token Economics

- **Max Supply**: 1,000,000,000 ARU (micro-ARU unit scale: $10^{-6}$ ARU)
- **Founder Allocation**: 1.5% (Vesting: 48-month linear, non-unlockable, no special governance privileges)
- **Premine**: 1.5% (Reserved for bootstraps, security audits, and testnet rewards)
- **Treasury**: 5% (Infrastructure and grant pool managed via on-chain governance)
- **Block Reward (Genesis)**: 25 ARU (70% PoW Miner / 25% PoS Validator / 5% Treasury split)
- **Halving Era**: Halves every 4 years (4,204,800 blocks per era)

---

## 📁 Repository Structure

The repository is structured as a Cargo workspace containing modular crates:

```
├── apps/                 # Wallets, dashboards, mobile miners
├── crates/
│   ├── ahash/            # AHash CPU/ARM-optimized ASIC-resistant mining pipeline
│   ├── consensus/        # Consensus validation coordinator & fork choice rules
│   ├── crypto/           # Ed25519 signing, public key hash derivation, and BLAKE3
│   ├── explorer/         # Indexer and REST API search explorer
│   ├── mempool/          # Memory pool transaction admission and Replace-by-Fee (RBF)
│   ├── networking/       # Length-prefixed TCP P2P networking and synchronization
│   ├── node/             # Node runtime coordinator, CLI commands, and RPC API server
│   ├── primitives/       # Addresses, nonces, blocks, headers, serialization helpers
│   ├── state/            # Account ledger state model
│   ├── storage/          # RocksDB transactional storage batch layer
│   └── wallet-cli/       # CLI wallet utilities and transaction signing
├── docs/                 # Architecture, specifications, and ADRs (ADR-XXXX)
├── infrastructure/       # Docker and deployment descriptors
├── scripts/              # Setup, utility, and maintenance scripts
└── tests/                # Conformance, Byzantine, and recovery integration test suites
```

---

## 🛠️ Official Development Environment (ODE)

To enforce strict determinism and compiler reproducibility, the workspace is aligned to a verified development stack.  
📖 See [**docs/ode.md**](docs/ode.md) for the complete specification, build matrix, and phase roadmap.

### Official Toolchain

| Tool | Version |
|---|---|
| OS | Ubuntu 24.04 LTS |
| Rust / Cargo | `1.96.0` |
| GCC | `14` |
| LLVM / Clang | `18` |
| CMake | `3.30+` |
| RocksDB | `8.10.x` |

### Official Build Matrix

| Environment | Status |
|---|---|
| GitHub Actions (Ubuntu 24.04) | ✅ Official |
| GitHub Codespaces | ✅ Official |
| VS Code Dev Container | ✅ Official |
| Docker Dev Image (`ghcr.io/aruna-network/dev`) | ✅ Official |
| Ubuntu Server 24.04 LTS | ✅ Official |
| Arch Linux | ⚡ Best Effort |
| Fedora | ⚡ Best Effort |
| macOS | 🔮 Future |
| Windows Native | 🔮 Future |

### VS Code Dev Container & Codespaces

Open in VS Code → **"Reopen in Container"** or launch in GitHub Codespaces.  
The container automatically provisions all compilers, sets required environment variables, and prefetches Cargo dependencies.

---

## ⚙️ Compilation & Testing

To compile or test the project locally, you must specify the include flag for standard headers required by RocksDB compilation on modern GCC systems:

### Build the Node
```bash
CXXFLAGS="-include cstdint" cargo build --release -p aruna-node
```

### Run Node in Daemon Mode
```bash
CXXFLAGS="-include cstdint" cargo run --release -p aruna-node -- daemon --p2p-port 9000 --rpc-port 8080
```

### Run Tests
To run the full suite of unit, integration, and Byzantine conformance tests:
```bash
CXXFLAGS="-include cstdint" cargo test --workspace
```

### Run Nightly Performance Benchmark
To measure execution TPS, latency, and memory footprint:
```bash
CXXFLAGS="-include cstdint" cargo run --release -p aruna-testing --bin performance_benchmark
```

---

## 🚢 Deployment

ARUNA uses a **three-image deployment strategy** to cleanly separate concerns across development, compilation, and production runtime.

### Image Architecture

| Image | Tag | Purpose |
|---|---|---|
| Development | `ghcr.io/aruna-network/dev` | VS Code, Cursor, Codespaces — full toolchain |
| Builder | `ghcr.io/aruna-network/builder` | Compiles release binary only |
| Runtime | `ghcr.io/aruna-network/node` | Minimal Ubuntu + binary + config. No Rust. |

---

### Method 1: One-Command Bootstrap (Recommended)

Clone the repository and run the bootstrap script. It will guide you through choosing Docker or native deployment:

```bash
git clone https://github.com/ARUNA-Network/aruna-core.git
cd aruna-core
./scripts/bootstrap.sh
```

Or directly specify the mode:
```bash
./scripts/bootstrap.sh docker    # Containerized via Docker Compose
./scripts/bootstrap.sh native    # Bare-metal with systemd service
```

---

### Method 2: Docker Compose

The fastest path to a running node — no Rust compiler required on host:

```bash
docker compose up -d --build
```

This will:
1. Compile the ARUNA node from source inside the builder stage.
2. Package only the binary and genesis config into a minimal runtime image.
3. Start the node daemon on `P2P :9000` and `RPC :8080`.
4. Persist ledger data in `./data_sumatera/` on your host machine.

```bash
# View running services
docker compose ps

# Stream logs
docker compose logs -f

# Stop the node
docker compose down
```

---

### Method 3: Dev Container (VS Code / Codespaces)

Open the project in VS Code and select **"Reopen in Container"**, or open directly in GitHub Codespaces. The Dev Container will automatically provision:
- GCC 14 + LLVM 18 + CMake
- Rust `1.96.0` toolchain
- Rust Analyzer, TOML, crates, and LLDB debugger extensions

Then build and run the node locally:
```bash
CXXFLAGS="-include cstdint" cargo run --release -p aruna-node -- daemon --p2p-port 9000 --rpc-port 8080
```

---

### Method 4: Bare-Metal systemd Service

For production VPS/server deployment on Ubuntu 24.04:

```bash
sudo ./scripts/install.sh
sudo systemctl start aruna-node
```

The install script will:
1. Install system build dependencies (GCC 14, LLVM 18, CMake, OpenSSL).
2. Compile and install the binary to `/usr/local/bin/aruna-node`.
3. Create a `aruna` system user and `/var/lib/aruna-node` data directory.
4. Register and enable `aruna-node.service` under systemd.

```bash
# Check node status
sudo systemctl status aruna-node

# Stream real-time logs
journalctl -u aruna-node -f
```

---

## 🛡️ Byzantine & Protocol Protections

ARUNA Network enforces zero-trust validation rules across network and ledger state layers:
1. **CPU/DOS protection**: Transactions are checked for memory duplicates *before* running CPU-heavy Ed25519 signature checks in the mempool.
2. **Sync range boundaries**: Sync requests are bounded to a maximum of 500 blocks; violation closes the TCP channel immediately.
3. **Double handshake protection**: Any duplicate handshake attempts over active connections result in peer disconnection.
4. **Duplicate block discard**: Blocks are verified against database headers *before* entering signature validation loops, saving CPU load.
5. **State Commitment**: Header commitments validate Merkle roots and state root updates deterministically.
