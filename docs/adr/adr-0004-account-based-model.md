# ADR-0004: Account-Based Model

## Status
Proposed

## Context
A public blockchain state model tracks balances and determines how transactions mutate the ledger. There are two primary paradigms:
1. **UTXO (Unspent Transaction Output):** Pioneered by Bitcoin. States are unspent coins that must be consumed in their entirety as inputs and recreated as new outputs.
2. **Account-Based Architecture:** Pioneered by Ethereum. The state is represented as a global ledger of accounts, each containing a balance, a nonce, optional contract code, and contract storage. State transitions are ledger balance transfers.

ARUNA Network aims to support standard smart contracts, full EVM equivalence, decentralized applications (such as native AMM DEX), and standard Web3 tooling (MetaMask, Hardhat, Ethers.js) to encourage ecosystem integration.

## Problem
While the UTXO model offers benefits in transaction parallelism and simplified validation, it introduces severe friction for smart contract development:
1. **EVM Compatibility:** EVM is built natively on an account-based architecture. Mapping EVM contract bytecodes to a UTXO ledger requires complex state models (like eUTXO) that break standard Ethereum developer tooling compatibility.
2. **State Concurrency Conflict:** In UTXO, any transaction modifying a contract state must consume the UTXO representing that contract. If multiple users try to interact with the same smart contract (e.g., swapping tokens on a DEX) in the same block, only one transaction can succeed. The others will fail due to double-spending inputs, creating user-experience issues.
3. **Complexity for Solo Founder:** Designing a custom eUTXO contract language and compiler requires significant development resources and time, violating the founder protection rule.

We need a state model that natively supports smart contracts, simplifies client integration, and retains full compatibility with existing Web3 libraries.

## Decision
We adopt the **Account-Based State Model** as the core ledger architecture for ARUNA.

### Account State Structure:
Each account in the state database contains:
* **Address:** The unique account identifier (Bech32m format).
* **Balance:** The quantity of native ARU coins owned by the account.
* **Nonce:** A counter incremented with each transaction sent from the account, preventing replay attacks.
* **Code (Optional):** The compiled EVM bytecode (if the account is a smart contract).
* **Storage (Optional):** A key-value database representing the internal variables and state of a smart contract.

### State Storage:
The global state mapping accounts to their balances and variables is stored in a structured database managed by **RocksDB**. State changes occur sequentially and deterministically on block execution.

## Alternatives
* **Alternative A: UTXO (Bitcoin-style):** Rejected because it is incompatible with EVM, making decentralized exchanges (AMMs) and MetaMask integration highly complex.
* **Alternative B: Hybrid UTXO/Account Model:** E.g., Cardano or Qtum. These models exist but introduce substantial development complexity, compiler overhead, and lack native Solidity/Foundry ecosystem integration.

## Consequences
* **Positive:**
  * **EVM Equivalence:** EVM contracts can be deployed and executed natively, allowing MetaMask, Ethers.js, and Hardhat to work out-of-the-box.
  * **Ecosystem Simplicity:** Standard wallets, block explorers, and indexers are much simpler to build and maintain for a small team.
  * **Shared State Access:** Multiple transactions can interact with the same contract storage in a single block (processed in a defined order within the block execution pipeline).
* **Negative:**
  * **State Bloat:** Over time, the size of the RocksDB state will grow as new accounts and smart contract storage mappings are created. Snapshots and state pruning will be required.
  * **Sequential Execution Constraint:** Transaction validation within a block is generally sequential to prevent state race conditions (though parallel optimistic execution can be researched in future phases).

## Migration
Not applicable. The account state model is the genesis design of the ledger.

## Security Impact
The account-based model requires careful transaction sequencing. Every transaction must be validated against the sender's account nonce and balance before execution to prevent double-spending and replay attacks. Reentrancy and state exhaustion threats must be defended against via strict smart contract gas and storage limits.
