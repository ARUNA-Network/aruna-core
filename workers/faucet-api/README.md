# ARUNA Faucet API (Cloudflare Worker)

## Overview
Stateless Edge worker for public testnet faucet token requests.

## Architecture
- **Sybil Protection**: Validates captcha tokens before executing transactions.
- **Edge Rate Limiting**: Employs Cloudflare KV for IP/address-based rate-limits.
- **Safe Signing**: Uses secure env-stored seed or KMS secrets to construct and sign faucet transfer transactions.
