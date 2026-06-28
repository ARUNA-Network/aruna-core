#!/usr/bin/env bash
# ARUNA Network — Node Updater Script
# Pulls latest image/code and performs a graceful rolling restart

set -euo pipefail

INSTALL_DIR="/opt/aruna-core"
COMPOSE_FILE="deployment/compose/docker-compose.production.yml"

echo "============================================"
echo "  ARUNA Network Node Updater               "
echo "============================================"

if [ "$EUID" -ne 0 ]; then
    echo "🚨 Please run as root: sudo bash deployment/scripts/update-node.sh"
    exit 1
fi

# Pull latest source
echo ""
echo "📥 Pulling latest changes..."
git -C "$INSTALL_DIR" fetch origin main
CHANGES=$(git -C "$INSTALL_DIR" log HEAD..origin/main --oneline)

if [ -z "$CHANGES" ]; then
    echo "✅ Already up to date."
else
    echo "📋 New commits:"
    echo "$CHANGES"
    echo ""
    read -rp "Apply update? (yes/no): " confirm
    [ "$confirm" = "yes" ] || { echo "Cancelled."; exit 0; }
    git -C "$INSTALL_DIR" pull origin main
fi

# Pull new Docker image
echo ""
echo "🐳 Pulling latest node image..."
docker compose -f "${INSTALL_DIR}/${COMPOSE_FILE}" pull --ignore-pull-failures

# Restart node gracefully (stop_grace_period: 30s handles RocksDB flush)
echo ""
echo "🔄 Restarting ARUNA node (graceful)..."
systemctl restart docker-compose-aruna.service

# Wait for healthy status
echo ""
echo "⏳ Waiting for node to become healthy..."
for i in $(seq 1 12); do
    STATUS=$(docker inspect --format='{{.State.Health.Status}}' aruna-node 2>/dev/null || echo "starting")
    if [ "$STATUS" = "healthy" ]; then
        echo "✅ Node is healthy!"
        break
    fi
    echo "  ($i/12) Status: $STATUS — waiting 5s..."
    sleep 5
done

echo ""
echo "============================================"
echo "  ✅ Update complete!"
echo "  journalctl -u docker-compose-aruna -f"
echo "============================================"
