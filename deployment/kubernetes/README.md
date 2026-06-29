# ARUNA Kubernetes Deployment Strategy

## Overview
Production-grade deployment orchestration for the ARUNA Network blockchain.

## Layout
- **Node StatefulSet**: Node validators deployed as a StatefulSet to persist RocksDB state.
- **Indexer Daemon**: Indexer run as a sidecar/daemon to sync block data into PostgreSQL.
- **Explorer API Deployment**: Auto-scaled stateless REST API deployment reading PostgreSQL.
