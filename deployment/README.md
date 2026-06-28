# ARUNA Node Deployment Guide

> **Target**: Ubuntu 24.04 LTS Server / VPS  
> **Stack**: systemd + Docker Compose + Cloudflare Tunnel

---

## Arsitektur

```
Internet
    │
    ▼
Cloudflare DNS + CDN
    │
    ▼
Cloudflare Tunnel (cloudflared.service)   ← systemd, always alive
    │                                        tidak terpengaruh docker restart
    ▼
http://localhost:8080
    │
    ▼
docker-compose-aruna.service              ← systemd, manages Docker Compose
    │
    └── aruna-node (container)
            │
            └── RocksDB (/opt/aruna-data)
```

**Mengapa cloudflared tidak dimasukkan ke Docker Compose?**

Jika cloudflared ada di dalam Compose dan kita menjalankan `docker compose down`, tunnel ikut mati. Padahal yang kita inginkan:

- Node restart → tunnel tetap hidup
- Saat node hidup kembali → RPC langsung bisa diakses tanpa intervensi manual

Dengan memisahkan keduanya ke systemd, keduanya saling independen.

---

## Struktur File

```
deployment/
├── systemd/
│   ├── cloudflared.service           ← tunnel (independen dari Docker)
│   └── docker-compose-aruna.service  ← mengawasi docker compose
├── compose/
│   └── docker-compose.production.yml ← node runtime (bersih, tanpa cloudflared)
├── monitoring/
│   ├── prometheus.yml
│   ├── grafana/
│   │   └── datasources.yml
│   └── docker-compose.monitoring.yml
├── scripts/
│   ├── install-node.sh    ← instalasi lengkap dari nol
│   ├── update-node.sh     ← update node ke versi terbaru
│   └── healthcheck.sh     ← cek status semua komponen
└── README.md              ← dokumen ini
```

---

## Prerequisites

- Ubuntu 24.04 LTS (VPS/Server)
- Akun Cloudflare dengan domain aktif
- Cloudflare Tunnel sudah dibuat di Cloudflare Dashboard → Zero Trust → Networks → Tunnels

---

## Quick Install (Satu Perintah)

```bash
git clone https://github.com/ARUNA-Network/aruna-core.git /opt/aruna-core
cd /opt/aruna-core
sudo bash deployment/scripts/install-node.sh
```

Script akan:
1. Install Docker
2. Install cloudflared
3. Minta Cloudflare Tunnel Token → simpan di `/etc/cloudflared/tunnel.env`
4. Install dan enable kedua systemd service
5. Start semua service

---

## Manual Setup (Step-by-Step)

### Step 1 — Install Docker

```bash
curl -fsSL https://get.docker.com | sh
systemctl enable docker && systemctl start docker
```

### Step 2 — Install cloudflared

```bash
curl -fsSL https://pkg.cloudflare.com/cloudflare-main.gpg \
    | gpg --dearmor -o /usr/share/keyrings/cloudflare-main.gpg
echo "deb [signed-by=/usr/share/keyrings/cloudflare-main.gpg] \
https://pkg.cloudflare.com/cloudflared $(lsb_release -cs) main" \
    > /etc/apt/sources.list.d/cloudflared.list
apt-get update && apt-get install cloudflared
```

### Step 3 — Konfigurasi Cloudflare Tunnel Token

```bash
mkdir -p /etc/cloudflared
# Salin token dari Cloudflare Dashboard → Zero Trust → Tunnels → pilih tunnel → Configure
echo "CLOUDFLARE_TUNNEL_TOKEN=eyJhIjoiXXXXXX..." > /etc/cloudflared/tunnel.env
chmod 600 /etc/cloudflared/tunnel.env
```

> **Tidak ada config.yml.** Hostname dan routing diatur sepenuhnya di **Cloudflare Dashboard**.  
> Tidak ada credentials file. Tidak ada tunnel ID hardcoded.

### Step 4 — Clone Repository

```bash
git clone https://github.com/ARUNA-Network/aruna-core.git /opt/aruna-core
mkdir -p /opt/aruna-data
```

### Step 5 — Install systemd Services

```bash
cp /opt/aruna-core/deployment/systemd/cloudflared.service /etc/systemd/system/
cp /opt/aruna-core/deployment/systemd/docker-compose-aruna.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable cloudflared docker-compose-aruna
systemctl start cloudflared docker-compose-aruna
```

---

## Manajemen Service

```bash
# Status semua komponen
bash /opt/aruna-core/deployment/scripts/healthcheck.sh

# Log node
journalctl -u docker-compose-aruna -f

# Log tunnel
journalctl -u cloudflared -f

# Restart node (tunnel tetap hidup)
systemctl restart docker-compose-aruna

# Restart tunnel
systemctl restart cloudflared

# Update ke versi terbaru
bash /opt/aruna-core/deployment/scripts/update-node.sh
```

---

## Firewall

```bash
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 9000/tcp  # P2P — wajib terbuka ke internet
# Port 8080 TIDAK dibuka — bound ke 127.0.0.1, hanya cloudflared yang mengakses
sudo ufw enable
```

---

## Verifikasi

```bash
# Dari host server
curl http://localhost:8080/status

# Dari internet (via Cloudflare Tunnel)
curl https://rpc.yourdomain.com/status
```

Keduanya harus mengembalikan JSON status node yang sama.

---

## Multi-VPS Setup

Setiap VPS menggunakan tunnel token yang berbeda dari Cloudflare Dashboard:

```
VPS 1                          VPS 2
────────────────────           ────────────────────
cloudflared.service            cloudflared.service
  token: TOKEN_VPS1              token: TOKEN_VPS2
  routing: rpc-1.aruna.network   routing: rpc-2.aruna.network
    │                              │
docker-compose-aruna           docker-compose-aruna
  aruna-node (port 9000)         aruna-node (port 9000)
```

Setiap tunnel dikonfigurasi di Cloudflare Dashboard secara terpisah. Tidak ada config.yml yang perlu disinkronkan antar VPS.

---

## Data & Backup

```
/opt/aruna-data/   ← blockchain data (bind mount dari container)
/opt/aruna-core/   ← source code dan compose files
/etc/cloudflared/  ← tunnel token (jangan backup ke tempat publik)
```

```bash
# Backup blockchain data
sudo tar -czf aruna-backup-$(date +%Y%m%d).tar.gz /opt/aruna-data/
```
