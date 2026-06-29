# Official Development Environment (ODE) — Policy & Usage

> **Authority**: This document is the single source of truth for the ARUNA Network development environment policy.
> **Enforcement**: All contributors, CI pipelines, and production deployments MUST comply with this policy.
> **ADR Reference**: ADR-0001 — Official Development Environment

---

## Policy Statement

**All official ARUNA Network development MUST occur inside the ODE.**

The ODE is defined as the Docker image `ghcr.io/aruna-network/ode:latest`, built from `docker/ode/Dockerfile`.

This is not a suggestion. It is a technical requirement enforced by the following rules:

---

## Enforcement Rules

### Rule 1 — All PRs Must Pass on the ODE

Every pull request to `main` is validated by GitHub Actions using `ubuntu-24.04`
with the exact toolchain versions defined in `docker/ode/Dockerfile`.

A PR will **not be merged** if it fails CI.

### Rule 2 — Bugs from Non-ODE Environments Are Not Priority

If a bug is reported from an environment that is not the ODE (e.g., macOS,
Fedora, Windows, a different GCC version), it will be tagged as **non-ODE** and
triaged as low-priority. The reporter is expected to reproduce the bug in the ODE
before it is assigned to a milestone.

### Rule 3 — No Toolchain Exceptions Without an ADR

Any proposal to change the toolchain (Rust version, GCC version, LLVM version,
base OS) requires a formal ADR with a migration path. Casual upgrades are not
permitted because they can silently break consensus-critical cryptographic code.

---

## GHCR Image Catalog

The following images are officially published and maintained:

| Image | Description | Who Uses It |
|-------|-------------|-------------|
| `ghcr.io/aruna-network/ode:latest` | Full build toolchain — Rust, GCC, Clang, LLVM, CMake, RocksDB deps | DevContainer, Codespaces, CI, local dev |
| `ghcr.io/aruna-network/node-builder:latest` | ODE + compiled `aruna-node` binary | Build system, artifact extraction |
| `ghcr.io/aruna-network/node-runtime:latest` | Minimal image: binary + config + TLS certs only | Production nodes, `docker-compose up` |

> **Size expectation:**
> - `ode` — large (~2–4 GB) — intended
> - `node-builder` — large (~2–4 GB) — intermediate, not run in production
> - `node-runtime` — small (<100 MB) — the only image deployed to production nodes

---

## Toolchain Specification

All versions are pinned in `docker/ode/Dockerfile` and audited on 2026-06-30.

| Tool | Version | Notes |
|------|---------|-------|
| **OS** | Ubuntu 24.04 LTS (Noble Numbat) | Only supported base |
| **Rust** | `1.96.0` | Pinned via `rust-toolchain.toml` |
| **GCC** | `14.2.0` (`gcc-14`) | Primary C/C++ compiler |
| **Clang** | `18.1.3` (`clang-18`) | Required for `libclang` |
| **LLVM** | `18.1.3` (`llvm-18`) | Required for proc-macros |
| **CMake** | `3.28.3` | Required for RocksDB build |
| **RocksDB** | `8.9.1` (`librocksdb-dev`) | State storage dependency |
| **OpenSSL** | `3.0.13` (`libssl-dev`) | TLS and crypto primitives |

> ⚠️ **Do NOT use GCC 15 or newer** — RocksDB headers fail to compile due to
> implicit `<cstdint>` removal. The workaround `CXXFLAGS="-include cstdint"` is
> applied automatically inside the ODE.

---

## Required Environment Variables

These variables MUST be set in all build contexts. They are set automatically
inside the ODE image, DevContainer, and `docker-compose.yml`.

```bash
export CC=gcc
export CXX=g++
export CXXFLAGS="-include cstdint"
export LIBCLANG_PATH=/usr/lib/llvm-18/lib
export CLANG_PATH=/usr/bin/clang
```

---

## How to Use the ODE

### Option A: VS Code Dev Container / GitHub Codespaces (Recommended)

```bash
git clone https://github.com/ARUNA-Network/aruna-core.git
# Open in VS Code → "Reopen in Container"
# Or press "Code" → "Open with Codespaces" on GitHub
```

The container automatically:
1. Builds the ODE image from `docker/ode/Dockerfile`
2. Runs `postCreateCommand.sh` — verifies tools and prefetches Cargo deps
3. Forwards ports `9000` (P2P) and `8080` (RPC)
4. Sets all required environment variables

### Option B: Pull the Pre-Built ODE Image

```bash
docker pull ghcr.io/aruna-network/ode:latest

docker run -it --rm \
  -v $(pwd):/workspace \
  -w /workspace \
  ghcr.io/aruna-network/ode:latest \
  bash
```

Inside the container, all tools are ready. Run builds directly:

```bash
cargo build --workspace
cargo test --workspace
cargo build --release -p aruna-node
```

### Option C: Docker Compose

```bash
# Start the interactive dev shell
docker compose run --rm dev bash

# Run the test suite
docker compose run --rm test

# Start a local node
docker compose up -d node
```

### Option D: Bare-Metal Ubuntu 24.04 LTS Only

> This option is **not recommended** for active development. Use it only for
> production server deployments.

```bash
sudo ./scripts/install.sh
sudo systemctl start aruna-node
```

---

## Build Matrix

| Environment | Status | Notes |
|-------------|--------|-------|
| GitHub Actions (Ubuntu 24.04) | ✅ **Official** | All PRs must pass |
| GitHub Codespaces | ✅ **Official** | Via `.devcontainer/` |
| VS Code Dev Container | ✅ **Official** | Via `.devcontainer/` |
| `ghcr.io/aruna-network/ode` (Docker) | ✅ **Official** | Validated on every push to `main` |
| Ubuntu Server 24.04 LTS (bare-metal) | ✅ **Official** | Production deployment only |
| Arch Linux | ⚡ Best Effort | GCC/LLVM versions may differ — bugs low priority |
| Fedora | ⚡ Best Effort | GCC/LLVM versions may differ — bugs low priority |
| macOS | 🔮 Future | Blocked by RocksDB + cross-compilation constraints |
| Windows Native | 🔮 Future | No ETA |

---

## Supported Compilation Targets

| Target | Status | Use Case |
|--------|--------|----------|
| `x86_64-unknown-linux-gnu` | ✅ Official | Servers, CI, VPS |
| `aarch64-unknown-linux-gnu` | ✅ Official | ARM64 servers, Raspberry Pi 4/5, ARM VPS |
| `aarch64-linux-android` | 🔮 Future | Android mining client |

---

## Verification Checklist

After setting up the ODE, verify the environment:

```bash
rustc --version    # rustc 1.96.0 (...)
cargo --version    # cargo 1.96.0 (...)
gcc --version      # gcc (Ubuntu 14.2.0-...) 14.2.x
clang --version    # Ubuntu clang version 18.1.3 (...)
cmake --version    # cmake version 3.28.3
```

Then run the full test suite:

```bash
cargo test --workspace
# Expected: all tests pass with 0 failures
```

---

## Package Version Re-Audit Process

APT package versions can change when Ubuntu publishes point releases. When re-auditing:

1. Run inside `ubuntu:24.04`:
   ```bash
   docker run --rm ubuntu:24.04 bash -c \
     "apt-get update -qq && apt-cache show <pkg> | grep '^Version:' | head -1"
   ```
2. Update the pinned version in `docker/ode/Dockerfile`
3. Update the pinned version in `docker/node/Dockerfile` (runtime stage)
4. Update the audit date comment in both Dockerfiles
5. Open a PR — CI will validate the new versions automatically

---

## CI Workflow Map

```
Push to main
│
├─ docker/ode/Dockerfile changed
│   └── docker-ode.yml
│       ├── build     → ghcr.io/aruna-network/ode:sha-<commit>
│       ├── validate  → rustc ✓ cargo ✓ gcc ✓ cmake ✓ clang ✓
│       │               cargo check --workspace ✓
│       │               cargo test --workspace ✓
│       └── push      → ghcr.io/aruna-network/ode:latest
│
└─ crates/** or Cargo.* changed
    └── docker-node.yml
        ├── node-builder  → ghcr.io/aruna-network/node-builder:sha-<commit>
        ├── node-runtime  → ghcr.io/aruna-network/node-runtime:sha-<commit>
        └── smoke-test    → binary exists ✓, no build toolchain in runtime ✓
```
