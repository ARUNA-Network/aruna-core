# =============================================================================
# ARUNA Network — Unified Node Dockerfile
#
# Build targets:
#   dev       — Full development environment (DevContainer, local CI)
#   builder   — Compiles the release binary (internal stage)
#   runtime   — Minimal production image
#
# Usage:
#   DevContainer / dev image  : docker build --target dev -t aruna-node-dev .
#   Production node image     : docker build -t ghcr.io/aruna-network/node:latest .
#
# ADR reference: docs/adr/ (Infrastructure / Docker)
# =============================================================================

# ── Stage 1: dev ─────────────────────────────────────────────────────────────
# Single source of truth for the build toolchain.
# Used directly by DevContainer and as the base for the builder stage.
FROM ubuntu:24.04 AS dev

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install compilation dependencies (GCC 14, LLVM 18, Clang 18, CMake, RocksDB)
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    gcc-14 \
    g++-14 \
    clang-18 \
    llvm-18 \
    libclang-18-dev \
    cmake \
    librocksdb-dev \
    libbz2-dev \
    zlib1g-dev \
    libsnappy-dev \
    liblz4-dev \
    libzstd-dev \
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

# Install pinned Rust toolchain
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain 1.96.0 --profile default

# Sanity-check: all required tools must be present
RUN rustc --version && cargo --version && gcc --version && cmake --version && clang --version


# ── Stage 2: builder ─────────────────────────────────────────────────────────
# Compiles the production binary on top of the dev stage.
# Not shipped — only the output binary is copied to the runtime stage.
FROM dev AS builder

WORKDIR /usr/src/aruna
COPY . .
RUN cargo build --release -p aruna-node


# ── Stage 3: runtime ─────────────────────────────────────────────────────────
# Minimal image shipped to production.
# Contains only the binary, config, and runtime libraries.
FROM ubuntu:24.04 AS runtime

ENV DEBIAN_FRONTEND=noninteractive

# Only runtime libraries — no build toolchain
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/aruna/target/release/aruna-node /app/aruna-node
COPY --from=builder /usr/src/aruna/config /app/config

EXPOSE 9000
EXPOSE 8080

ENTRYPOINT ["/app/aruna-node"]
CMD ["--p2p-port", "9000", "--rpc-port", "8080"]
