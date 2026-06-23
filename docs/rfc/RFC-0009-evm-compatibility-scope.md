# RFC-0009: EVM Integration & Compatibility Scope

## 1. Background
This RFC outlines the scope of EVM compatibility in the ARUNA execution layer, focusing on tool integration, signature handling, and execution gas limits.

## 2. Proposed Specification

### A. Execution Equivalence
* **Target:** Strict Ethereum execution equivalence (London or Shanghai hard fork baseline).
* **Opcode Support:** 100% of standard EVM opcodes must execute identically.
* **Tooling:** Supported out-of-the-box: MetaMask, Solidity, Hardhat, Foundry, Ethers.js.

### B. Dual Signature Engine
* **Wallet Signatures:** Standard transfers and staking use **Ed25519** signatures.
* **EVM Signatures:** Deploying and interacting with EVM contracts requires **secp256k1** signatures to ensure MetaMask compatibility.
* **Encoding:** Block transaction list checks the signature type byte (`0` for Ed25519, `1` for secp256k1) and applies the correct validation algorithm.

### C. Gas Boundaries
* **Block Gas Limit:** Capped at **30,000,000 gas** to prevent CPU exhaustion on ARM nodes.
