#!/usr/bin/env bash

# ARUNA Network Cloudflare Platform Deployment Script
# Automates deployment of Edge Workers and Cloudflare Pages static frontends.

set -euo pipefail

echo "=== Starting ARUNA Platform Deployment ==="

# 1. Verify wrangler CLI installation
if ! command -v wrangler &> /dev/null; then
    echo "Error: wrangler CLI is not installed. Run 'npm install -g wrangler'."
    exit 1
fi

# 2. Deploy Edge Workers
WORKERS=(
    "workers/rpc-gateway"
    "workers/dashboard-api"
    "workers/faucet-api"
)

for worker in "${WORKERS[@]}"; do
    echo "--- Deploying Edge Worker: $worker ---"
    if [ -d "$worker" ]; then
        (cd "$worker" && wrangler deploy)
    else
        echo "Warning: Directory $worker not found, skipping."
    fi
done

# 3. Publish Static Frontends to Cloudflare Pages
# Assumes Wrangler Pages project is configured for each UI directory
PAGES=(
    "apps/explorer-ui"
    "apps/faucet"
)

for app in "${PAGES[@]}"; do
    echo "--- Publishing Pages App: $app ---"
    if [ -d "$app" ]; then
        # Publish build assets to CF Pages under project name matching dir name
        PROJECT_NAME=$(basename "$app")
        wrangler pages deploy "$app" --project-name="$PROJECT_NAME"
    else
        echo "Warning: Directory $app not found, skipping."
    fi
done

echo "=== ARUNA Platform Deployment Completed Successfully ==="
