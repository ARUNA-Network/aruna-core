# ADR-0014: BIP39 + HD Wallet Standard

## Status
Proposed

## Context
A cryptocurrency wallet manages the user's private keys, public keys, and corresponding blockchain addresses. Users must be able to back up, recover, and manage their keys easily.

The industry-standard approach uses hierarchical deterministic (HD) derivation, where a single seed phrase generates an infinite tree of private/public key pairs. This removes the need to make frequent file backups of individual private keys.

## Problem
If the wallet system does not follow recognized key management standards, several issues arise:
1. **Vendor Lock-in:** If the wallet uses a proprietary derivation mechanism, users cannot import their seed phrase into other wallet software, increasing dependency on a single app.
2. **Key Exposure:** If private keys are stored in plaintext on the device, mobile malware can steal them.
3. **Address Reuse:** Reusing the same address for all transactions reduces user privacy, as anyone can trace their entire transaction history on-chain.

We need a standardized, recoverable key management system that keeps keys local and is compatible with mainstream hardware and software standards.

## Decision
We enforce the combination of **BIP39 (Mnemonic Codes)**, **BIP32 (Hierarchical Deterministic Wallets)**, and **BIP44 (Multi-Account Derivation Paths)** as the official key management standards for ARUNA Network.

### 1. Seed Phrase Standard (BIP39)
* Wallets must support both 12-word and 24-word seed phrases.
* **24-word seed phrases** are recommended as the default option during wallet creation to maximize entropy (256-bit security).
* Mnemonics must be generated using cryptographically secure random number generators (CSPRNG).

### 2. Hierarchical Deterministic Derivation (BIP44)
We reserve a specific BIP44 coin type for the ARUNA Network:
* **Coin Type:** **`7777`**
* **Derivation Path Format:**
  `m / 44' / 7777' / account' / change / address_index`
* **Standard Default Path:**
  `m/44'/7777'/0'/0/0` (Primary address for standard wallets).

### 3. Local Execution & Key Protection
* Plaintext private keys or seed phrases must never be stored on disk, logged in debug consoles, or transmitted over the P2P or RPC network.
* **Mobile security:** Private keys must be generated and stored inside hardware-isolated keystores (Android Keystore / iOS Secure Enclave).
* **Local Signing:** All transaction payloads must be signed locally on the user's device. The node RPC interface must only accept fully signed transaction bytes for propagation (`transaction_send`).

## Alternatives
* **Alternative A: Raw Private Key Storage (Keystore JSON files):** Encrypted JSON files containing raw private keys. This was rejected because users frequently lose JSON backup files, and typing passwords on mobile devices is a poor user experience compared to seed phrases and biometrics.
* **Alternative B: Custom Derivation Path (e.g. reusing Ethereum's coin type 60'):** Reusing Ethereum's path makes metamask integration easier. However, it causes coin collision in multichain wallets (where standard wallets automatically display ARUNA balances as Ethereum balances), creating confusion. We use coin type `7777` to maintain network separation, while EVM compatibility layers will support secp256k1 keys.

## Consequences
* **Positive:**
  * **Interoperability:** Users can recover their funds on any BIP39-compliant wallet by importing their seed phrase.
  * **Privacy:** BIP44 allows generating change addresses automatically for every transaction, preventing address reuse.
  * **Security:** Hardware keystore storage isolates private keys from malware access.
* **Negative:**
  * Generating keys on mobile devices requires platform-specific Flutter channel integrations to access the Android Keystore natively.

## Migration
Not applicable. BIP39/44 is the standard from day zero.

## Security Impact
Local transaction signing ensures that even if an RPC node or the block validator is compromised, they cannot access user funds or signatures. The 256-bit entropy of 24-word seed phrases protects the wallet from brute-force dictionary attacks.
