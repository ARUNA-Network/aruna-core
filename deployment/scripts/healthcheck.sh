#!/usr/bin/env bash
# ARUNA Network — Node Health Check Script
# Can be used by monitoring systems or cron jobs

set -euo pipefail

RPC_URL="http://localhost:8080"
EXIT_CODE=0

echo "============================================"
echo "  ARUNA Network Node Health Check          "
echo "============================================"

# ── systemd service status ───────────────────────────────────────────────────
echo ""
echo "📋 systemd Services:"

CF_STATUS=$(systemctl is-active cloudflared.service 2>/dev/null || echo "inactive")
DC_STATUS=$(systemctl is-active docker-compose-aruna.service 2>/dev/null || echo "inactive")

[ "$CF_STATUS" = "active" ] && echo "  ✅ cloudflared.service: $CF_STATUS" \
    || { echo "  ❌ cloudflared.service: $CF_STATUS"; EXIT_CODE=1; }

[ "$DC_STATUS" = "active" ] && echo "  ✅ docker-compose-aruna.service: $DC_STATUS" \
    || { echo "  ❌ docker-compose-aruna.service: $DC_STATUS"; EXIT_CODE=1; }

# ── Docker container health ──────────────────────────────────────────────────
echo ""
echo "🐳 Container Status:"
DOCKER_STATUS=$(docker inspect --format='{{.State.Status}} ({{.State.Health.Status}})' aruna-node 2>/dev/null || echo "not found")
echo "  aruna-node: $DOCKER_STATUS"

if echo "$DOCKER_STATUS" | grep -q "unhealthy\|not found"; then
    EXIT_CODE=1
fi

# ── RPC endpoint ─────────────────────────────────────────────────────────────
echo ""
echo "🌐 RPC Endpoint ($RPC_URL/status):"
if curl -sf --max-time 5 "${RPC_URL}/status" > /tmp/aruna_status.json 2>/dev/null; then
    echo "  ✅ RPC responding"
    cat /tmp/aruna_status.json
else
    echo "  ❌ RPC not responding"
    EXIT_CODE=1
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo "============================================"
if [ "$EXIT_CODE" -eq 0 ]; then
    echo "  ✅ All checks passed"
else
    echo "  ❌ One or more checks FAILED"
fi
echo "============================================"

exit $EXIT_CODE
