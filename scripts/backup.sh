#!/usr/bin/env bash
# ARUNA Network — Blockchain Data Backup Script
# Creates a timestamped compressed archive of the node data directory

set -euo pipefail

BACKUP_DIR="./backups"
DATA_DIR="./data"
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
BACKUP_FILE="${BACKUP_DIR}/aruna-data-${TIMESTAMP}.tar.gz"

echo "========================================="
echo "   ARUNA Network Node Backup"
echo "========================================="

# Verify data directory exists
if [ ! -d "$DATA_DIR" ]; then
    echo "🚨 Error: Data directory '$DATA_DIR' not found."
    echo "Is the node initialized? Run 'docker compose up -d node' first."
    exit 1
fi

mkdir -p "$BACKUP_DIR"

echo "📦 Creating backup..."
echo "  Source:      $DATA_DIR"
echo "  Destination: $BACKUP_FILE"

# Stop node gracefully before backup to ensure RocksDB consistency
echo ""
echo "⏸️  Stopping node gracefully (stop_grace_period: 30s)..."
docker compose stop node 2>/dev/null || true

# Create compressed archive
tar -czf "$BACKUP_FILE" -C "$(dirname "$DATA_DIR")" "$(basename "$DATA_DIR")"

# Restart node
echo "▶️  Restarting node..."
docker compose start node 2>/dev/null || true

BACKUP_SIZE=$(du -sh "$BACKUP_FILE" | cut -f1)
echo ""
echo "========================================="
echo "✅ Backup complete!"
echo "   File: $BACKUP_FILE"
echo "   Size: $BACKUP_SIZE"
echo "========================================="
