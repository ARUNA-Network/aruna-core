#!/usr/bin/env bash
# ARUNA Network — Dev Container Post-Create Setup Script
# Runs automatically after the container is first created.
# Ensures the development environment is fully ready for coding.

set -euo pipefail

echo "=========================================="
echo "  ARUNA Network Dev Container Initialized"
echo "=========================================="

# ── Toolchain verification ──────────────────────────────────────────────────
echo ""
echo "🔍 Verifying Official Development Environment (ODE) toolchain..."

RUST_VERSION=$(rustc --version 2>/dev/null || echo "NOT FOUND")
CARGO_VERSION=$(cargo --version 2>/dev/null || echo "NOT FOUND")
GCC_VERSION=$(gcc --version 2>/dev/null | head -1 || echo "NOT FOUND")
CLANG_VERSION=$(clang --version 2>/dev/null | head -1 || echo "NOT FOUND")
CMAKE_VERSION=$(cmake --version 2>/dev/null | head -1 || echo "NOT FOUND")

echo "  🦀 Rust:    $RUST_VERSION"
echo "  📦 Cargo:   $CARGO_VERSION"
echo "  🔧 GCC:     $GCC_VERSION"
echo "  🔧 Clang:   $CLANG_VERSION"
echo "  🏗️  CMake:   $CMAKE_VERSION"

# ── Rust component setup ────────────────────────────────────────────────────
echo ""
echo "🧩 Installing required Rust components..."
rustup component add clippy rustfmt rust-src 2>/dev/null || true
rustup target add aarch64-unknown-linux-gnu 2>/dev/null || true

# ── Pre-fetch dependencies ──────────────────────────────────────────────────
echo ""
echo "📦 Pre-fetching Cargo workspace dependencies (this may take a moment)..."
export CXXFLAGS="-include cstdint"
cargo fetch

# ── Git config defaults ─────────────────────────────────────────────────────
echo ""
echo "🔧 Configuring git defaults..."
git config --global --add safe.directory /workspaces/aruna-core 2>/dev/null || true
git config --global core.autocrlf false 2>/dev/null || true

# ── Final summary ───────────────────────────────────────────────────────────
echo ""
echo "=========================================="
echo "  ✅ Dev Container is Ready!"
echo "------------------------------------------"
echo "  Quick commands:"
echo ""
echo "  Build:     CXXFLAGS=\"-include cstdint\" cargo build --release -p aruna-node"
echo "  Test:      CXXFLAGS=\"-include cstdint\" cargo test --workspace"
echo "  Run node:  CXXFLAGS=\"-include cstdint\" cargo run --release -p aruna-node -- daemon --p2p-port 9000 --rpc-port 8080"
echo "  Lint:      cargo clippy --workspace"
echo "  Format:    cargo fmt --all"
echo ""
echo "  Official Toolchain: Rust 1.96.0 | GCC 14 | LLVM 18 | CMake 3.30+"
echo "=========================================="
