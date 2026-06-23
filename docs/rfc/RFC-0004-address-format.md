# RFC-0004: Address Format & Checksum Specifications

## 1. Background
Addresses represent public keys. This RFC specifies the hashing, encoding, network separator formats, and error-correction checksum details.

## 2. Proposed Specification

### A. Key Derivation Flow
* **Input:** Raw Public Key (Ed25519 or secp256k1).
* **Hash:**
  $$\text{PubKeyHash} = \text{RIPEMD160}(\text{SHA256}(\text{PublicKey}))$$
* **Length:** 20 bytes.

### B. Bech32m base32 Encoding
* **Alphabet:** `qpzry9x8gf2tvdw0s3jn54khce6mua7l`
* **Separator:** `1`
* **Prefixes (HRP):**
  * Standard Wallets: `jaw1` (Mainnet), `sum1` (Sumatera), `kal1` (Kalimantan), `sul1` (Sulawesi), `pap1` (Papua).
  * Contracts: `jawc1`, `sumc1`, `kalc1`, `sulc1`, `papc1`.
* **Checksum:** BIP-350 standard BCH checksum computed over base32 indices.

### C. Address Validation Constraints
* All addresses must be lowercase.
* Address decoding libraries must verify the BCH checksum. If any error is detected, the transaction is rejected.
