#!/usr/bin/env bash
# ARUNA Network Node Installer Script
# Targets: Ubuntu 24.04 LTS (ODE alignment)

set -euo pipefail

echo "========================================="
echo "   ARUNA Network Core Node Installer   "
echo "========================================="

# 1. Root verification
if [ "$EUID" -ne 0 ]; then
    echo "🚨 Error: Please run this installer as root (using sudo)."
    exit 1
fi

# 2. OS verification
if [ -f /etc/os-release ]; then
    . /etc/os-release
    if [ "$ID" != "ubuntu" ] || [ "$VERSION_ID" != "24.04" ]; then
        echo "⚠️ Warning: This script is optimized for Ubuntu 24.04 LTS."
        echo "Continuing anyway..."
    fi
else
    echo "⚠️ Warning: Could not detect OS version."
fi

# 3. Build dependencies installation
echo "📦 Installing system build dependencies..."
apt-get update
apt-get install -y \
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
    && rm -rf /var/lib/apt/lists/*

# Set GCC 14 and Clang 18 as defaults
update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-14 100 --force
update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-14 100 --force
update-alternatives --install /usr/bin/clang clang /usr/bin/clang-18 100 --force
update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-18 100 --force

# 4. Rust toolchain check/install
export CC=gcc
export CXX=g++
export CLANG_PATH=/usr/bin/clang
export LIBCLANG_PATH=/usr/lib/llvm-18/lib
export CXXFLAGS="-include cstdint"

if ! command -v rustc &> /dev/null; then
    echo "🦀 Installing Rust toolchain (1.96.0)..."
    export RUSTUP_HOME=/usr/local/rustup
    export CARGO_HOME=/usr/local/cargo
    export PATH=/usr/local/cargo/bin:$PATH
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.96.0 --profile default
else
    echo "🦀 Rust is already installed: $(rustc --version)"
fi

# 5. Compile release binary
echo "🏗️ Compiling ARUNA Node in release mode..."
cargo build --release -p aruna-node

# 6. Install binary
echo "💾 Installing binary to /usr/local/bin..."
cp target/release/aruna-node /usr/local/bin/aruna-node
chmod +x /usr/local/bin/aruna-node

# 7. Create system user and directory structure
echo "👤 Creating system user and folder structure..."
if ! id -u aruna &>/dev/null; then
    groupadd -r aruna
    useradd -r -g aruna -d /var/lib/aruna-node -s /sbin/nologin -c "ARUNA daemon account" aruna
fi

mkdir -p /var/lib/aruna-node/config
cp config/genesis.sumatera.toml /var/lib/aruna-node/config/genesis.sumatera.toml
chown -R aruna:aruna /var/lib/aruna-node

# 8. Register systemd service
echo "⚙️ Registering systemd service..."
cp infrastructure/systemd/aruna-node.service /etc/systemd/system/aruna-node.service
systemctl daemon-reload
systemctl enable aruna-node.service

echo "========================================="
echo "🎉 Installation Complete!"
echo "-----------------------------------------"
echo "To start the node, run:"
echo "  sudo systemctl start aruna-node"
echo ""
echo "To check node status & logs, run:"
echo "  sudo systemctl status aruna-node"
echo "  journalctl -u aruna-node -f"
echo "========================================="
