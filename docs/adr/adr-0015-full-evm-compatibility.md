# ADR-0015: Full EVM Compatibility

## Status
Proposed

## Context
Decentralized applications (dApps) are built using smart contract frameworks. The dominant standard in the blockchain ecosystem is the Ethereum Virtual Machine (EVM), which executes Solidity compiler bytecodes. The EVM has a massive developer ecosystem, standard libraries (such as OpenZeppelin), and widely adopted tooling (Foundry, Hardhat, MetaMask, Ethers.js).

To enable builders to deploy dApps on ARUNA without rewriting their codebases, the network execution layer must support EVM execution.

## Problem
Attempting to modify the EVM or introducing custom execution parameters can cause issues:
1. **Broken Tooling:** Any divergence in EVM opcode execution, gas costs, or JSON-RPC responses will break compatibility with tools like Hardhat, Foundry, and MetaMask.
2. **Security Vulnerabilities:** Custom virtual machines or modified EVM engines often introduce security risks, such as reentrancy loopholes or compiler execution errors.
3. **State Exhaustion (DoS):** Unbounded contract execution can deplete node memory or disk space, crashing the validator and bringing down the P2P network.

We need a virtual machine execution engine that provides strict EVM equivalence while enforcing resource safety limits on commodity hardware.

## Decision
We implement a **Full EVM Compatibility Layer** as part of our execution engine (`crates/execution`).

### Core Integration Principles:
1. **Opcode Equivalence:** The execution layer must execute standard EVM bytecodes natively. No opcodes will be modified, and gas consumption costs must match standard Ethereum specifications.
2. **secp256k1 Signature Support:** In addition to the primary Ed25519 signature scheme, the consensus and transaction layers must support secp256k1 signatures specifically for EVM interactions, enabling MetaMask compatibility.
3. **EVM Security Constraints:**
   To prevent state exhaustion and Denial of Service (DoS) attacks on low-cost ARM/Android nodes, we enforce strict resource limits at the protocol level:
   * **Gas Limits:** Strict, non-overrideable block gas limits.
   * **Memory Limits:** The EVM memory allocation size per contract execution is capped.
   * **Storage Limits:** Capped maximum storage writes per transaction.
4. **EVM Contract Address Format:**
   Addresses representing smart contracts use the Bech32m format with a distinct contract identifier prefix (e.g., `jawc1...` on Mainnet). The node translates these prefixes internally to their raw 20-byte hexadecimal hashes during EVM execution.

## Alternatives
* **Alternative A: Custom Rust-WASM Virtual Machine:** Prioritize WebAssembly for smart contract execution. WASM is faster and supports multiple languages (Rust, C, Go). We rejected this because the developer ecosystem and Web3 tooling for WASM are fragmented, violating our ease-of-integration target.
* **Alternative B: EVM-compatible wrapper (Sidechain architecture):** Rejected because running the EVM as a separate sidechain increases bridging complexity and latency, violating our simple architecture principles.

## Consequences
* **Positive:**
  * **Ecosystem Compatibility:** Developers can deploy standard Solidity contracts (e.g., Uniswap v2 AMM, ERC-20 tokens, DAO governance) directly on ARUNA.
  * **MetaMask Support:** Users can connect MetaMask directly to ARUNA RPC nodes.
  * **Low Development Friction:** The solo founder can utilize existing, audited Solidity templates (like OpenZeppelin) for DEX and naming services.
* **Negative:**
  * Enforcing full EVM compatibility requires running dual signature support (Ed25519 for P2P/Validators, secp256k1 for EVM transactions), increasing transaction payload validation logic complexity.
  * The state engine must maintain compatibility between account balances and EVM contract storage structures.

## Migration
Not applicable. The EVM compatibility layer is scheduled for integration in Phase 4 (Papua Release Candidate) according to the roadmap.

## Security Impact
EVM contract sandboxing is strictly enforced. No smart contract execution can mutate consensus rules, validator registry stakes, or treasury governance balances directly. This prevents privilege escalation vulnerabilities.
