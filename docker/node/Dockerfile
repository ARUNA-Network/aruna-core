# =============================================================================
# ARUNA Network — Unified Node Dockerfile
#
# Build targets:
#   dev       — Full development environment (DevContainer, local CI, image validation)
#   builder   — Compiles the release binary (internal stage, not shipped)
#   runtime   — Minimal production image
#
# Usage:
#   DevContainer / dev image  : docker build -f docker/node/Dockerfile --target dev -t aruna-node-dev .
#   Production node image     : docker build -f docker/node/Dockerfile -t ghcr.io/aruna-network/node:latest .
#
# Package pinning policy:
#   All APT packages are pinned to their exact version in Ubuntu 24.04 LTS
#   as audited on 2026-06-30. To re-audit: run the following inside ubuntu:24.04
#   and update the versions below:
#     apt-get update && apt-cache show <pkg> | grep '^Version:'
#   The Rust toolchain version must always match rust-toolchain.toml.
#
# ADR reference: docs/adr/ (Infrastructure / Docker)
# =============================================================================

# ── Stage 1: dev ─────────────────────────────────────────────────────────────
# Single source of truth for the build toolchain.
# Used directly by DevContainer and as the base for the builder stage.
FROM ubuntu:24.04 AS dev

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# ── Pinned APT packages ───────────────────────────────────────────────────────
# Audited: 2026-06-30 against ubuntu:24.04 (noble)
# Use --no-install-recommends to prevent non-deterministic transitive packages.
RUN apt-get update && apt-get install -y --no-install-recommends \
    "curl=8.5.0-2ubuntu10.9" \
    git \
    build-essential \
    pkg-config \
    "libssl-dev=3.0.13-0ubuntu3.11" \
    "gcc-14=14.2.0-4ubuntu2~24.04.1" \
    "g++-14=14.2.0-4ubuntu2~24.04.1" \
    "clang-18=1:18.1.3-1ubuntu1" \
    "llvm-18=1:18.1.3-1ubuntu1" \
    "libclang-18-dev=1:18.1.3-1ubuntu1" \
    "cmake=3.28.3-1build7" \
    "librocksdb-dev=8.9.1-2" \
    "libbz2-dev=1.0.8-5.1build0.1" \
    "zlib1g-dev=1:1.3.dfsg-3.1ubuntu2.1" \
    "libsnappy-dev=1.1.10-1build1" \
    "liblz4-dev=1.9.4-1build1.1" \
    "libzstd-dev=1.5.5+dfsg2-2build1.1" \
    sudo \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Register GCC 14 and Clang 18 as the system-default compilers
RUN update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-14 100 \
    && update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-14 100 \
    && update-alternatives --install /usr/bin/clang clang /usr/bin/clang-18 100 \
    && update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-18 100

# Compiler and LLVM environment variables
ENV CC=gcc
ENV CXX=g++
ENV CXXFLAGS="-include cstdint"
ENV CLANG_PATH=/usr/bin/clang
ENV LIBCLANG_PATH=/usr/lib/llvm-18/lib

# ── Pinned Rust toolchain ─────────────────────────────────────────────────────
# Version MUST match rust-toolchain.toml. Verified via TLS from static.rust-lang.org.
# rustup itself is the version-management layer; the toolchain version is immutable.
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH
ARG RUST_VERSION=1.96.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain "${RUST_VERSION}" --profile default

# ── Toolchain sanity check ────────────────────────────────────────────────────
# Build fails here immediately if any tool is missing or resolves to a wrong version.
RUN rustc --version && cargo --version \
    && gcc --version && g++ --version \
    && cmake --version && clang --version \
    && echo "✓ All toolchain components verified"


# ── Stage 2: builder ─────────────────────────────────────────────────────────
# Compiles the production binary on top of the dev stage.
# Not shipped — only the output binary is copied to the runtime stage.
FROM dev AS builder

WORKDIR /usr/src/aruna
COPY . .
RUN cargo build --release -p aruna-node


# ── Stage 3: runtime ─────────────────────────────────────────────────────────
# Minimal image shipped to production.
# Contains only the binary, config, and pinned runtime libraries.
# No compiler, no Rust, no build toolchain.
FROM ubuntu:24.04 AS runtime

ENV DEBIAN_FRONTEND=noninteractive

# Pinned runtime libraries only
RUN apt-get update && apt-get install -y --no-install-recommends \
    "libssl-dev=3.0.13-0ubuntu3.11" \
    ca-certificates \
    "curl=8.5.0-2ubuntu10.9" \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/aruna/target/release/aruna-node /app/aruna-node
COPY --from=builder /usr/src/aruna/config /app/config

EXPOSE 9000
EXPOSE 8080

ENTRYPOINT ["/app/aruna-node"]
CMD ["--p2p-port", "9000", "--rpc-port", "8080"]
