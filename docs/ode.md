# Official Development Environment (ODE)

> **Authority**: This document is the single source of truth for ARUNA Network's build environment.  
> **Enforcement**: All contributors, CI pipelines, and production deployments MUST match this specification.  
> **ADR Reference**: ADR-0001 — Official Development Environment

---

## Rationale

ARUNA Network is a deterministic blockchain protocol. For consensus correctness, the code that runs on every node must behave identically. This requires the compiler, system libraries, and toolchain to be strictly identical across all environments.

The ODE eliminates the problem of "works on my machine" at the infrastructure level.

---

## Official Toolchain Versions

| Tool | Official Version | Notes |
|---|---|---|
| **Operating System** | Ubuntu 24.04 LTS (Noble Numbat) | Required for GCC 14 + LLVM 18 support |
| **Rust** | `1.96.0` | Pinned via `rust-toolchain.toml` |
| **Cargo** | `1.96.0` | Bundled with Rust 1.96.0 |
| **GCC** | `14` (`gcc-14` / `g++-14`) | Primary C/C++ compiler |
| **LLVM / Clang** | `18` (`clang-18` / `llvm-18`) | Required for `libclang` (proc-macros) |
| **CMake** | `3.30+` | Required for RocksDB build system |
| **RocksDB** | `8.10.x` | Bundled via `librocksdb-sys` crate |
| **OpenSSL** | `3.x` | System package `libssl-dev` |
| **Protobuf** | `3.x` | Reserved for future RPC expansion |

> ⚠️ **Do NOT use GCC 15 or newer** — RocksDB C++ headers fail to compile due to implicit `<cstdint>` changes.  
> The workaround `CXXFLAGS="-include cstdint"` is applied automatically in all official environments.

---

## Required Environment Variables

These variables MUST be set in all build contexts:

```bash
export CC=gcc
export CXX=g++
export CXXFLAGS="-include cstdint"
export LIBCLANG_PATH=/usr/lib/llvm-18/lib
export CLANG_PATH=/usr/bin/clang
```

In Dev Container and Docker environments, these are set automatically via `remoteEnv` and `ENV` directives.

---

## Official Build Matrix

| Environment | Status | Notes |
|---|---|---|
| GitHub Actions (Ubuntu 24.04) | ✅ **Official** | Primary CI — all PRs must pass |
| GitHub Codespaces | ✅ **Official** | Via `.devcontainer/` |
| VS Code Dev Container | ✅ **Official** | Via `.devcontainer/` |
| Docker Dev Image (`ghcr.io/aruna-network/dev`) | ✅ **Official** | Auto-published on `main` push |
| Ubuntu Server 24.04 LTS (VPS/Bare-metal) | ✅ **Official** | Via `scripts/install.sh` |
| Arch Linux | ⚡ Best Effort | GCC/LLVM versions may differ |
| Fedora | ⚡ Best Effort | GCC/LLVM versions may differ |
| macOS (ARM or Intel) | 🔮 Future | Blocked by RocksDB compatibility |
| Windows Native | 🔮 Future | No ETA |

---

## Supported Compilation Targets

| Target | Status | Use Case |
|---|---|---|
| `x86_64-unknown-linux-gnu` | ✅ Official | Servers, CI, VPS |
| `aarch64-unknown-linux-gnu` | ✅ Official | ARM64 servers, Raspberry Pi 4/5, ARM VPS |
| `aarch64-linux-android` | 🔮 Future | Android mining client |

---

## Phase Roadmap

### Phase 1 — Official Development Environment ✅ Complete
- [x] `.devcontainer/Dockerfile` — Full toolchain container
- [x] `.devcontainer/devcontainer.json` — VS Code / Codespaces config
- [x] `.devcontainer/postCreateCommand.sh` — Auto-setup on container creation
- [x] `.devcontainer/features.json` — Machine-readable toolchain specification
- [x] `rust-toolchain.toml` — Toolchain pinned to `1.96.0`
- [x] `Dockerfile.dev` — Standalone dev image
- [x] `Dockerfile.node` — Multi-stage minimal runtime image
- [x] `docker-compose.yml` — One-command node bootstrap
- [x] `infrastructure/systemd/aruna-node.service` — Systemd service
- [x] `scripts/install.sh` — Bare-metal installer
- [x] `scripts/bootstrap.sh` — One-command bootstrap selector

### Phase 2 — Continuous Integration ✅ Complete
- [x] `.github/workflows/ci.yml` — Full test suite on every PR (Ubuntu 24.04)
- [x] `.github/workflows/bench.yml` — Nightly performance benchmarks
- [x] `.github/workflows/docker-publish.yml` — Auto-publish dev image to GHCR

### Phase 3 — Continuous Development (In Progress)
- [ ] Publish `ghcr.io/aruna-network/dev:latest` to GHCR
- [ ] Verify Codespaces end-to-end setup works
- [ ] Add `CONTRIBUTING.md` with new-developer onboarding guide
- [ ] Add `EditorConfig` for universal code style enforcement

### Phase 4 — Production Deployment
- [ ] Publish `ghcr.io/aruna-network/node:latest` runtime image
- [ ] Add multi-arch build matrix (`amd64` + `arm64`) in CI
- [ ] Add automated release pipeline with semantic versioning
- [ ] Publish ARM64-optimized binaries as GitHub Releases assets

---

## Setup Instructions

### Option A: VS Code Dev Container / Codespaces (Recommended)

```bash
git clone https://github.com/ARUNA-Network/aruna-core.git
# Open in VS Code → "Reopen in Container"
# Or open in GitHub Codespaces
```

The container will automatically:
1. Build the ODE image from `.devcontainer/Dockerfile`
2. Run `.devcontainer/postCreateCommand.sh` to verify tools and prefetch dependencies
3. Forward ports `9000` (P2P) and `8080` (RPC)
4. Set all required environment variables

### Option B: Docker Dev Image

```bash
docker pull ghcr.io/aruna-network/dev:latest

docker run -it --rm \
  -v $(pwd):/workspace \
  -w /workspace \
  ghcr.io/aruna-network/dev:latest \
  bash
```

Then inside the container:
```bash
CXXFLAGS="-include cstdint" cargo build --release -p aruna-node
```

### Option C: Docker Compose (Node)

```bash
docker compose up -d --build
```

### Option D: Bare-Metal Ubuntu 24.04

```bash
sudo ./scripts/install.sh
sudo systemctl start aruna-node
```

---

## Verification

After setup, verify the environment with:

```bash
rustc --version    # should print: rustc 1.96.0 (...)
cargo --version    # should print: cargo 1.96.0 (...)
gcc --version      # should contain: 14.x.x
clang --version    # should contain: 18.x.x
cmake --version    # should contain: 3.30+
```

Then run the full test suite:
```bash
CXXFLAGS="-include cstdint" cargo test --workspace
# Expected: all tests pass with 0 failures
```
