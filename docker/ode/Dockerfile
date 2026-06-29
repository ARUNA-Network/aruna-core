# =============================================================================
# ARUNA Network — Official Development Environment (ODE) Image
#
# Image: ghcr.io/aruna-network/ode:latest
#
# Purpose:
#   Full build toolchain for local development, DevContainer, Codespaces,
#   and CI. This is the ONLY officially supported environment for ARUNA
#   Network development. See docs/ode.md for policy.
#
# Usage:
#   docker build -f docker/ode/Dockerfile -t ghcr.io/aruna-network/ode:latest .
#   docker run -it --rm -v $(pwd):/workspace ghcr.io/aruna-network/ode:latest bash
#
# Package pinning policy:
#   All APT packages are pinned to their exact version in Ubuntu 24.04 LTS
#   as audited on 2026-06-30. To re-audit run inside ubuntu:24.04:
#     apt-get update && apt-cache show <pkg> | grep '^Version:'
#   The Rust toolchain version MUST always match rust-toolchain.toml.
#
# ADR reference: docs/adr/ADR-0001 — Official Development Environment
# =============================================================================

FROM ubuntu:24.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# ── Pinned APT packages ───────────────────────────────────────────────────────
# Audited: 2026-06-30 against ubuntu:24.04 (noble)
# --no-install-recommends prevents non-deterministic transitive packages.
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
# CXXFLAGS=-include cstdint is required — GCC 14+ dropped implicit <cstdint> inclusion
ENV CC=gcc
ENV CXX=g++
ENV CXXFLAGS="-include cstdint"
ENV CLANG_PATH=/usr/bin/clang
ENV LIBCLANG_PATH=/usr/lib/llvm-18/lib

# ── Pinned Rust toolchain ─────────────────────────────────────────────────────
# RUST_VERSION must match rust-toolchain.toml — the project source of truth.
# Override at build time: docker build --build-arg RUST_VERSION=x.y.z
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH
ARG RUST_VERSION=1.96.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain "${RUST_VERSION}" --profile default

# ── Toolchain sanity check ────────────────────────────────────────────────────
# Build fails here immediately if any required tool is missing or incorrect.
RUN rustc --version && cargo --version \
    && gcc --version && g++ --version \
    && cmake --version && clang --version \
    && echo "✓ ODE toolchain verified"

WORKDIR /workspace
