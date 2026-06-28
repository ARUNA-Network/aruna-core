#!/usr/bin/env bash
# ARUNA Network Core One-Command Bootstrap Script
# Enforces the Official Development Environment (ODE) standards

set -euo pipefail

echo "========================================="
echo "   ARUNA Network Core Node Bootstrap    "
echo "========================================="

# Detect argument or prompt user
MODE=${1:-""}

if [ -z "$MODE" ]; then
    echo "Please choose a deployment mode:"
    echo "  1) Docker Compose (Recommended - containerized)"
    echo "  2) Bare-metal / systemd Service (Ubuntu 24.04)"
    read -rp "Enter choice [1-2]: " choice
    case "$choice" in
        1) MODE="docker" ;;
        2) MODE="native" ;;
        *) echo "Invalid choice"; exit 1 ;;
    esac
fi

if [ "$MODE" = "docker" ]; then
    echo "🐳 Bootstrapping ARUNA node via Docker Compose..."
    if ! command -v docker &> /dev/null; then
        echo "🚨 Error: docker is not installed. Please install Docker first."
        exit 1
    fi
    
    # Trigger docker-compose builds and start daemon services
    if docker compose version &> /dev/null; then
        docker compose up -d --build
    else
        docker-compose up -d --build
    fi
    
    echo "========================================="
    echo "🎉 Node is starting in containerized background!"
    echo "To view status and logs, run:"
    echo "  docker compose ps"
    echo "  docker compose logs -f"
    echo "========================================="
    
elif [ "$MODE" = "native" ]; then
    echo "🖥️ Bootstrapping ARUNA node natively..."
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
    chmod +x "${SCRIPT_DIR}/install.sh"
    sudo "${SCRIPT_DIR}/install.sh"
else
    echo "🚨 Error: unknown mode '$MODE'. Use 'docker' or 'native'."
    exit 1
fi
