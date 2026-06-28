#!/usr/bin/env bash
# ARUNA Network — Full Node Installation Script
# Target: Ubuntu 24.04 LTS
# Architecture:
#   systemd → cloudflared.service (tunnel, always alive)
#   systemd → docker-compose-aruna.service → Docker Compose → aruna-node

set -euo pipefail

REPO_URL="https://github.com/ARUNA-Network/aruna-core.git"
INSTALL_DIR="/opt/aruna-core"
DATA_DIR="/opt/aruna-data"

echo "============================================"
echo "  ARUNA Network Node — Full Installation   "
echo "  Target: Ubuntu 24.04 LTS                 "
echo "============================================"

# ── Root check ──────────────────────────────────────────────────────────────
if [ "$EUID" -ne 0 ]; then
    echo "🚨 Please run as root: sudo bash deployment/scripts/install-node.sh"
    exit 1
fi

# ── 1. System packages ───────────────────────────────────────────────────────
echo ""
echo "📦 [1/6] Installing system packages..."
apt-get update -qq
apt-get install -y -qq curl git ca-certificates gnupg lsb-release

# ── 2. Docker ────────────────────────────────────────────────────────────────
echo ""
echo "🐳 [2/6] Installing Docker..."
if ! command -v docker &>/dev/null; then
    curl -fsSL https://get.docker.com | sh
    systemctl enable docker
    systemctl start docker
    echo "✅ Docker installed: $(docker --version)"
else
    echo "✅ Docker already installed: $(docker --version)"
fi

# ── 3. cloudflared ───────────────────────────────────────────────────────────
echo ""
echo "☁️  [3/6] Installing cloudflared..."
if ! command -v cloudflared &>/dev/null; then
    curl -fsSL https://pkg.cloudflare.com/cloudflare-main.gpg \
        | gpg --dearmor -o /usr/share/keyrings/cloudflare-main.gpg
    echo "deb [signed-by=/usr/share/keyrings/cloudflare-main.gpg] \
https://pkg.cloudflare.com/cloudflared $(lsb_release -cs) main" \
        > /etc/apt/sources.list.d/cloudflared.list
    apt-get update -qq
    apt-get install -y -qq cloudflared
    echo "✅ cloudflared installed: $(cloudflared --version)"
else
    echo "✅ cloudflared already installed: $(cloudflared --version)"
fi

# ── 4. Clone repository ───────────────────────────────────────────────────────
echo ""
echo "📥 [4/6] Setting up ARUNA repository..."
if [ ! -d "$INSTALL_DIR" ]; then
    git clone "$REPO_URL" "$INSTALL_DIR"
    echo "✅ Cloned to $INSTALL_DIR"
else
    git -C "$INSTALL_DIR" pull origin main
    echo "✅ Updated existing repo at $INSTALL_DIR"
fi

mkdir -p "$DATA_DIR"

# ── 5. Systemd services ───────────────────────────────────────────────────────
echo ""
echo "⚙️  [5/6] Installing systemd services..."

# cloudflared tunnel token
mkdir -p /etc/cloudflared

if [ ! -f /etc/cloudflared/tunnel.env ]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Cloudflare Tunnel Setup"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  1. Buka Cloudflare Dashboard → Zero Trust → Networks → Tunnels"
    echo "  2. Buat tunnel baru atau pilih tunnel yang ada"
    echo "  3. Salin token tunnel Anda (format: eyJhIjoiXXX...)"
    echo ""
    read -rp "  Paste Cloudflare Tunnel Token: " TUNNEL_TOKEN
    echo "CLOUDFLARE_TUNNEL_TOKEN=${TUNNEL_TOKEN}" > /etc/cloudflared/tunnel.env
    chmod 600 /etc/cloudflared/tunnel.env
    echo "✅ Token saved to /etc/cloudflared/tunnel.env"
else
    echo "✅ /etc/cloudflared/tunnel.env already exists"
fi

# Install systemd units
cp "${INSTALL_DIR}/deployment/systemd/cloudflared.service" /etc/systemd/system/
cp "${INSTALL_DIR}/deployment/systemd/docker-compose-aruna.service" /etc/systemd/system/

# Patch WorkingDirectory to match INSTALL_DIR
sed -i "s|WorkingDirectory=.*|WorkingDirectory=${INSTALL_DIR}|" \
    /etc/systemd/system/docker-compose-aruna.service

systemctl daemon-reload
systemctl enable cloudflared.service
systemctl enable docker-compose-aruna.service

# ── 6. Start services ────────────────────────────────────────────────────────
echo ""
echo "🚀 [6/6] Starting services..."
systemctl start cloudflared.service
systemctl start docker-compose-aruna.service

sleep 5

# ── Summary ──────────────────────────────────────────────────────────────────
echo ""
echo "============================================"
echo "  ✅ Installation Complete!"
echo "--------------------------------------------"
echo "  Architecture:"
echo "    systemd → cloudflared.service  (tunnel, always alive)"
echo "    systemd → docker-compose-aruna.service → aruna-node"
echo ""
echo "  Commands:"
echo "    systemctl status cloudflared"
echo "    systemctl status docker-compose-aruna"
echo "    journalctl -u cloudflared -f"
echo "    journalctl -u docker-compose-aruna -f"
echo ""
echo "  Verify:"
echo "    curl http://localhost:8080/status"
echo "    curl https://your-hostname/status"
echo "============================================"
