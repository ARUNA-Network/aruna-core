#!/usr/bin/env bash
# ARUNA Network — Node Update Script
# Pulls latest source, rebuilds the node image, and restarts with zero data loss

set -euo pipefail

echo "========================================="
echo "   ARUNA Network Node Updater"
echo "========================================="

# 1. Pull latest changes from main branch
echo "📥 Pulling latest changes from origin/main..."
git fetch origin main
git diff --stat HEAD origin/main

read -rp "Proceed with update? (yes/no): " confirm
if [ "$confirm" != "yes" ]; then
    echo "Update cancelled."
    exit 0
fi

git pull origin main

# 2. (Optional) create backup before update
read -rp "Create backup before updating? (yes/no): " do_backup
if [ "$do_backup" = "yes" ]; then
    chmod +x ./scripts/backup.sh
    ./scripts/backup.sh
fi

# 3. Rebuild node image
echo ""
echo "🏗️  Building new node image..."
docker compose build node

# 4. Restart node with new image (graceful stop — 30s flush)
echo ""
echo "🔄 Restarting node with new image..."
docker compose stop node
docker compose up -d node

# 5. Wait for health check
echo ""
echo "⏳ Waiting for node to become healthy..."
for i in $(seq 1 12); do
    STATUS=$(docker inspect --format='{{.State.Health.Status}}' aruna-node 2>/dev/null || echo "unknown")
    if [ "$STATUS" = "healthy" ]; then
        echo "✅ Node is healthy!"
        break
    fi
    echo "  ($i/12) Status: $STATUS — waiting 5s..."
    sleep 5
done

echo ""
echo "========================================="
echo "✅ Update complete!"
echo "   Use 'docker compose logs -f node' to monitor."
echo "========================================="
