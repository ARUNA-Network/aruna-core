#!/usr/bin/env bash
# ARUNA Network — Blockchain Data Restore Script
# Restores blockchain data from a timestamped backup archive

set -euo pipefail

BACKUP_FILE="${1:-}"
DATA_DIR="./data"

echo "========================================="
echo "   ARUNA Network Node Restore"
echo "========================================="

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: ./scripts/restore.sh <backup-file>"
    echo ""
    echo "Available backups:"
    ls -lh ./backups/*.tar.gz 2>/dev/null || echo "  (no backups found in ./backups/)"
    exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
    echo "🚨 Error: Backup file '$BACKUP_FILE' not found."
    exit 1
fi

echo "📦 Restore from: $BACKUP_FILE"
echo ""
echo "⚠️  WARNING: This will overwrite the current blockchain data in '$DATA_DIR'."
read -rp "Continue? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Cancelled."
    exit 0
fi

# Stop node before restore
echo ""
echo "⏸️  Stopping node..."
docker compose stop node 2>/dev/null || true

# Back up current data just in case
if [ -d "$DATA_DIR" ]; then
    SAFETY_BACKUP="./backups/pre-restore-$(date +%Y-%m-%d_%H-%M-%S).tar.gz"
    mkdir -p ./backups
    echo "💾 Safety backup of current data → $SAFETY_BACKUP"
    tar -czf "$SAFETY_BACKUP" -C "$(dirname "$DATA_DIR")" "$(basename "$DATA_DIR")" 2>/dev/null || true
    rm -rf "$DATA_DIR"
fi

# Restore from archive
echo "📂 Restoring data..."
mkdir -p "$(dirname "$DATA_DIR")"
tar -xzf "$BACKUP_FILE" -C "$(dirname "$DATA_DIR")"

# Restart node
echo "▶️  Starting node..."
docker compose start node 2>/dev/null || true

echo ""
echo "========================================="
echo "✅ Restore complete!"
echo "   Data restored to: $DATA_DIR"
echo "========================================="
