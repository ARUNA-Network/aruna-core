# ARUNA NETWORK

## MASTER PRD

### Volume 6 — Wallet, Addressing, Cryptography & Key Management

---

# DESIGN PHILOSOPHY

The wallet system must be:
* Secure
* Recoverable
* Human Friendly
* Mobile Friendly
* Future Proof

*Users should not need deep technical knowledge to safely store ARU.*

---

# CRYPTOGRAPHIC FOUNDATIONS

ARUNA uses battle-tested cryptography.
* **Rule:** Never invent custom cryptographic algorithms.

### Primary Signature Algorithm:
**Ed25519**
* *Reasons:* Fast verification, small signatures, excellent Rust ecosystem, mobile friendly, widely audited.

### Future/EVM Support:
**secp256k1** for EVM compatibility.

---

# ACCOUNT MODEL

Account-Based Architecture. Each account contains:
* Address
* Balance
* Nonce
* Optional Contract Code
* Optional Storage

---

# ADDRESS FORMAT

Addresses use **Bech32m Encoding**.
* *Benefits:* Human readable, error detection, QR friendly, mobile friendly.

### Network Prefixes:
ARUNA uses Indonesian region-based prefixes to indicate the network/stage:
* **Sumatera Testnet:** `sum1abc...`
* **Kalimantan Testnet:** `kal1abc...`
* **Sulawesi Testnet:** `sul1abc...`
* **Papua Release Candidate:** `pap1abc...`
* **Jawa Mainnet:** `jaw1abc...`

### Address Structure:
`[prefix][encoded public key hash]`
* *Example:* `jaw1k3y8m0f3n2x...`

---

# ADDRESS TYPES

* **Standard Wallet:** Transfer, Receive, Stake, Vote.
* **Validator Wallet:** Validator Operations, Staking, Governance.
* **Treasury Wallet:** Protocol Treasury, controlled through governance.
* **Contract Wallet:** Smart Contracts.

---

# SEED PHRASE & HD WALLET

* **Standard:** BIP39 (12 or 24 words). **24 words** is recommended for maximum security.
* **HD Derivation Standard:** BIP32 & BIP44.
* **Derivation Path:** `m/44'/7777'/0'/0/0`
  * Derivation path constant **`7777`** is reserved for ARUNA.
* *Private keys never leave the device.*

---

# WALLET TYPES

* **Mobile Wallet:** Flutter (Android, iOS future).
* **Desktop Wallet:** Flutter (Linux, Windows, macOS).
* **CLI Wallet:** Rust (for Developers, Validators, and Infrastructure Operators).

---

# MOBILE SECURITY MODEL

* Private keys must be stored using:
  * **Android Keystore**
  * **iOS Secure Enclave** (future)
* **Features:** Biometric unlock, PIN protection, device encryption.
* **Transaction Signing:** All signing occurs locally. Private keys are never transmitted.

---

# RECOVERY & BACKUP

* **Recovery Method:** Seed phrase only. If the device is lost, install the wallet, enter the seed phrase, and restore funds.
* *No central recovery service, no account reset, and no password recovery.*
* **Backup Policy:** The wallet must encourage offline backup (paper or metal backups). Screenshots, cloud notes, or messaging apps must be discouraged.

---

# MULTI-SIGNATURE SUPPORT

* **Future Version:** Native Multi-signature support.
* **Use Cases:** Treasury management, organization security, and DAOs.

---

# INTEGRATIONS

* **Staking:** Natively supports stake, unstake, delegate, and validator selection.
* **Governance:** Natively supports proposal voting, treasury voting, and protocol voting.
* **DEX (Future):** Integrated swaps, liquidity provision, and launchpad access.

---

# SECURITY & ANTI-PHISHING REQUIREMENTS

* Plaintext private keys must never be stored.
* Seed phrases must never be transmitted.
* Secrets must never be logged.
* Keys must never be exposed through APIs.
* **Anti-Phishing Features:** Address checksums, address verification prompts, transaction previews, and suspicious contract warnings.

---

# SUCCESS CRITERIA

The wallet system succeeds when:
1. Non-technical users can use it safely.
2. Android users can participate securely.
3. Recovery is 100% reliable.
4. Keys remain local and protected.
5. EVM compatibility is fully supported.
6. Addresses are immediately recognizable as ARUNA network.

End of Volume 6.
