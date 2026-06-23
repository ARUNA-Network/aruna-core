# ARUNA NETWORK

## MASTER PRD

### Volume 7 — Smart Contract, EVM & Execution Layer

---

# EXECUTION PHILOSOPHY

ARUNA must provide:
* Predictable Execution
* Deterministic Execution
* Secure Execution
* Low-Cost Execution

*Execution behavior must remain identical across Android, ARM64, x86_64, Linux, and Windows. No platform-specific execution differences are allowed.*

---

# EXECUTION MODEL

ARUNA adopts an **Account-Based State Model** with an **EVM Compatibility Layer**.

### Execution Flow:
```
Transaction → Mempool → Validation → Execution → State Transition → Block Inclusion
```

---

# EVM STRATEGY

ARUNA targets an **Ethereum Equivalent Environment**.
* **Supported Tooling:** MetaMask, Solidity, Hardhat, Foundry, Ethers.js, Web3.js.
* **Smart Contract Languages:** Solidity (primary), Vyper (future), Rust-WASM (under research).

---

# CONTRACT ADDRESS FORMAT

Contract accounts utilize the following prefix format:
* **Structure:** `[network_prefix]c1[encoded public key hash]`
* *Example (Jawa Mainnet):* `jawc1f3n9...` (where `jaw` is the Mainnet prefix and `c` indicates a contract account).

---

# EXECUTION ENGINE & GAS MODEL

* **Responsibilities:** Transaction execution, smart contract execution, state updates, gas consumption, and receipt generation.
* **Native Gas Token:** ARU.
* **Gas Philosophy:** Execution consumes Gas to prevent infinite loops, spam, and state abuse. Gas fees must remain significantly lower than Ethereum while sustaining validator costs.

---

# TRANSACTION TYPES

Supported transactions include:
1. **Native Transfer:** ARU Transfer.
2. **Staking:** Stake & Unstake ARU, Delegate Validator.
3. **Governance:** Proposal Voting.
4. **Smart Contracts:** Contract Deployment & Contract Execution.
5. **Ecosystem:** DEX Swaps.

---

# STATE TRANSITIONS & CONTRACT STORAGE

* **State Transitions:** Strict deterministic execution: `Previous State → Execution → New State`.
* **Contract Storage:** Persisted in the RocksDB State Layer (contains variables, mappings, balances, governance state, and contract state).
* **Receipt System:** Every execution generates a receipt containing: Transaction Hash, Block Height, Gas Used, Execution Status, Logs, and Events.
* **Event System:** Emitted events are utilized by wallets, explorers, indexers, DEX, and other dApps.

---

# TOKEN STANDARDS

* **Fungible Token Standard:** **ARC-20** (ARUNA Request for Comment), fully compatible with ERC-20 concepts.
  * *Required methods:* `name()`, `symbol()`, `decimals()`, `balanceOf()`, `transfer()`, `approve()`, `allowance()`, `transferFrom()`.
* **NFT Standard (Future):** **ARC-721** (Not MVP).

---

# NATIVE DEX ARCHITECTURE

A native decentralized exchange (DEX) is included in the ecosystem:
* **Model:** Automated Market Maker (AMM) using the Constant Product Formula: \(x \cdot y = k\).
* **Initial Pairs:** `ARU` / `ARC-20`.
* **DEX Fee:** 0.30% per trade.
  * *Distribution:* 70% to Liquidity Providers, 20% to Treasury, 10% to Protocol Reserve.

---

# FUTURE ECOSYSTEM FOUNDATIONS

* **Launchpad:** Native token and liquidity creation, vesting, and fundraising, using ARU for launch fees.
* **Domain Service:** Native naming service (e.g. `example.aruna` resolving to addresses). Registration fees paid in ARU are routed to the Treasury.

---

# EVM SECURITY RULES

* Never allow unchecked execution or protocol privilege escalation.
* No contract execution can override consensus rules or treasury controls.
* **Resource Limits:** Enforce gas limits, memory limits, execution limits, and storage limits to prevent state exhaustion, denial of service, and infinite execution.
* **Treasury Controls:** All Treasury activities must be controlled through Governance contracts (not private founder keys).

---

# SUCCESS CRITERIA

The execution layer succeeds when:
1. MetaMask works out-of-the-box.
2. Solidity contracts compile and run seamlessly.
3. Native DEX and governance operate correctly.
4. Gas remains cheap and predictable.
5. Execution remains 100% deterministic across ARM64, x86_64, and Android.

End of Volume 7.
