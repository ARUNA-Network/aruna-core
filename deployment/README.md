# ARUNA Node Deployment Guide

> Target: Ubuntu 24.04 LTS Server / VPS  
> Stack: Docker + Docker Compose + Cloudflare Tunnel

---

## Architecture

```
Internet
    │
    ▼
Cloudflare DNS
    │
    ▼
Cloudflare Tunnel Container (cloudflared)
    │
    ▼
http://node:8080   ← internal Docker network
    │
    ▼
ARUNA Node Container (node)
    │
    ▼
RocksDB (./data/)
```

Port `9000` (P2P) dibuka langsung ke internet. Port `8080` (RPC) **tidak** dibuka ke publik — hanya diakses secara internal oleh cloudflared.

---

## Prerequisites

- Docker + Docker Compose v2
- Cloudflare account dengan domain aktif
- Cloudflare Tunnel sudah dibuat (`cloudflared tunnel create aruna-rpc`)

---

## Step 1 — Clone Repository

```bash
git clone https://github.com/ARUNA-Network/aruna-core.git
cd aruna-core
```

---

## Step 2 — Konfigurasi Cloudflare Tunnel

### 2a. Install cloudflared

```bash
curl -fsSL https://pkg.cloudflare.com/cloudflare-main.gpg | sudo tee /usr/share/keyrings/cloudflare-main.gpg >/dev/null
echo 'deb [signed-by=/usr/share/keyrings/cloudflare-main.gpg] https://pkg.cloudflare.com/cloudflared focal main' | sudo tee /etc/apt/sources.list.d/cloudflared.list
sudo apt-get update && sudo apt-get install cloudflared
```

### 2b. Login dan buat tunnel

```bash
cloudflared tunnel login
cloudflared tunnel create aruna-rpc
```

Catat **Tunnel ID** yang diberikan (format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`).

### 2c. Buat config.yml

Salin template dan isi dengan Tunnel ID Anda:

```bash
cp deployment/config.example.yml ~/.cloudflared/config.yml
nano ~/.cloudflared/config.yml
```

Isi sesuai Tunnel ID dan hostname Anda:

```yaml
tunnel: YOUR_TUNNEL_ID_HERE
credentials-file: /etc/cloudflared/YOUR_TUNNEL_ID_HERE.json

ingress:
  - hostname: rpc.yourdomain.com
    service: http://node:8080

  - service: http_status:404
```

> ⚠️ **Penting**: Jangan commit `config.yml` ke Git. File ini sudah ada di `.gitignore`.

### 2d. Daftarkan DNS

```bash
cloudflared tunnel route dns aruna-rpc rpc.yourdomain.com
```

---

## Step 3 — Jalankan Node

### Build image

```bash
docker compose build node
```

### Jalankan node + tunnel

```bash
docker compose --profile production up -d
```

### Cek status

```bash
docker compose ps
docker compose logs -f node
docker compose logs -f cloudflared
```

---

## Step 4 — Verifikasi

### Dari host:

```bash
curl http://localhost:8080/status
```

Harus mengembalikan JSON status node.

### Dari internet:

```bash
curl https://rpc.yourdomain.com/status
```

Harus mengembalikan JSON yang sama.

---

## Management Commands

| Perintah | Fungsi |
|---|---|
| `docker compose --profile production up -d` | Start semua service |
| `docker compose down` | Stop semua service |
| `docker compose logs -f node` | Stream log node |
| `docker compose logs -f cloudflared` | Stream log tunnel |
| `docker compose restart node` | Restart node |
| `docker compose pull` | Update image ke versi terbaru |

---

## Backup & Restore

```bash
# Backup blockchain data
./scripts/backup.sh

# Restore dari backup
./scripts/restore.sh ./backups/aruna-data-2026-06-29.tar.gz

# Update node ke versi terbaru
./scripts/update-node.sh
```

---

## Monitoring (Opsional)

Untuk mengaktifkan Prometheus + Grafana:

```bash
docker compose -f deployment/monitoring/docker-compose.monitoring.yml up -d
```

Akses Grafana di `http://localhost:3000` (admin/admin).

---

## Firewall

Pastikan hanya port yang diperlukan yang terbuka:

```bash
sudo ufw allow 9000/tcp   # P2P networking — wajib terbuka ke internet
sudo ufw deny  8080/tcp   # RPC — ditangani cloudflared, jangan buka ke publik
sudo ufw allow 22/tcp     # SSH
sudo ufw enable
```

---

## Struktur Data

```
./data/                  ← blockchain data (bind mount)
~/.cloudflared/          ← cloudflare tunnel credentials (jangan commit!)
  config.yml             ← tunnel config (gitignored)
  *.json                 ← tunnel credentials (gitignored)
./backups/               ← hasil script backup.sh
```
