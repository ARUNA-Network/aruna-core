# ARUNA NETWORK

## MASTER PRD

### Volume 3 — Protocol Architecture & Consensus

---

# PROTOCOL DESIGN PHILOSOPHY

ARUNA is designed around:
* Simplicity
* Determinism
* Security
* Accessibility
* Long-Term Sustainability

*The protocol must remain understandable, auditable, and maintainable by a small team.*

---

# CHAIN MODEL

ARUNA uses:
**Account-Based Architecture** (Not UTXO).

### Reasons:
* EVM Compatibility
* Simpler Wallet Experience
* Native Smart Contracts
* DEX Support
* Lower Development Complexity

---

# STATE MODEL

Each account contains:
* Address
* Balance
* Nonce
* Code (Optional)
* Storage (Optional)

State is stored in:
**RocksDB**

*State transitions must be deterministic.*

---

# CONSENSUS MODEL

Hybrid Consensus (70% Proof of Work, 25% Proof of Stake, 5% Treasury)

### Consensus Goals:
* Community Mining
* ARM Accessibility
* Economic Sustainability
* Decentralization

---

# AHASH SPECIFICATION V1

AHash is the native mining algorithm.

### Objectives:
* CPU Focused
* ARM Friendly
* Android Friendly
* Energy Conscious
* Cache Efficient
* Memory Hard

### Pipeline:
1. **Block Header**
2. **BLAKE3**
3. **AES Mixing Stage**
4. **Argon2 Memory Expansion**
5. **ARM NEON Optimization Layer**
6. **Final Digest**

### Target Hardware:
Android, ARM64, Raspberry Pi, Mini PC, ARM Server, x86

---

# BLOCK PRODUCTION

* **Target Block Time:** 30 Seconds
* **Target Finality:** 2 Minutes
* **Recommended Confirmations:** 4 Blocks (Soft Finality)

---

# BLOCK STRUCTURE

### Block Header:
* Version
* Previous Hash
* Merkle Root
* Timestamp
* Difficulty Target
* Nonce
* Validator Signature Root
* Treasury Root
* AHash Result

### Block Body:
* Transactions
* Validator Metadata
* Treasury Metadata
* Network Metadata

---

# BLOCK SIZE

* **Initial Block Limit:** 2 MB
* **Dynamic Adjustment:** Future Governance Controlled

---

# TRANSACTION MODEL

### Transaction Types:
* Transfer
* Stake
* Unstake
* Validator Registration
* Governance Vote
* Contract Deployment
* Contract Execution
* DEX Operations

---

# TRANSACTION FEE MODEL

* All transactions require fees.
* **Fee Asset:** ARU
* **Fee Burning:** No
* **Fee Distribution:** Validators, Miners, Treasury
* *Future governance may modify fee allocation.*

---

# VALIDATOR MODEL

* **Minimum Stake:** 10,000 ARU
* **Validator Requirements:** Full Node, Stable Connectivity, Blockchain Synchronization, State Verification.

### Validator Responsibilities:
* Validate Blocks
* Verify Transactions
* Maintain Network Availability
* Participate in Governance
* Maintain State Consistency

### Validator Offline Policy:
* **No Slashing**
* Offline Validator: Receives no rewards, can rejoin later, no stake destruction.

### Delegated Staking:
* Users may delegate stake.
* **Delegators:** Maintain ownership.
* **Validators:** Receive commission.

---

# NODE TYPES

* **Seed Node:** Peer Discovery & Bootstrapping.
* **Full Node:** Store Full Chain, Verify State, Broadcast Blocks.
* **Validator Node:** Consensus Participation, Validate, Sign, Govern.
* **Archive Node:** Historical Storage, Serve Explorer & Analytics.
* **RPC Node:** Application/Wallet Queries, Developer Access.
* **Indexer Node:** Transaction/Address Indexing, Explorer Support.

---

# NETWORKING LAYER

* **Protocol:** libp2p
* **Features:** Peer Discovery, Block Propagation, Transaction Propagation, Network Synchronization, Fork Detection.
* **Transport:** TCP, QUIC (Future)

---

# MEMPOOL DESIGN

* **Purpose:** Temporary Transaction Storage
* **Rules:** Validate Before Entry, Reject Invalid/Duplicate Transactions, Prioritize Higher Fees.

---

# DIFFICULTY ADJUSTMENT

* **Goal:** Maintain 30-second block production.
* **Adjustment Interval:** Every Block
* **Algorithm:** Weighted Moving Average
* **Inputs:** Previous Difficulty, Block Production Time, Network Hashrate
* **Objectives:** Prevent Oscillation & Difficulty Shock, Maintain Stability

---

# FORK CHOICE RULE

### Selection Priority:
1. Valid Chain
2. Highest Accumulated Work (Most Work Chain)
3. Highest Finalized Stake Weight
4. Earliest Network Acceptance

---

# FINALITY MODEL

* **Soft Finality:** 4 Blocks
* **Hard Finality:** Validator Confirmation
* **Target:** ~2 Minutes

---

# TREASURY INTEGRATION

* Treasury rewards generated automatically at the protocol level.
* No manual minting, no founder control, no hidden issuance.

---

# EVM COMPATIBILITY

* **Target:** Full Compatibility
* **Supported Tools:** MetaMask, Solidity, Hardhat, Foundry, Web3.js, Ethers.js
* **Future Goal:** Near Drop-In Ethereum Compatibility

---

# SECURITY PRINCIPLES

* Never Trust Clients
* Always Verify Signatures
* Always Verify State
* Always Verify Consensus
* Always Verify Stake
* Always Verify Block Validity

---

# SUCCESS CRITERIA

The protocol succeeds when:
* Android miners can participate.
* ARM devices remain competitive.
* Validators remain accessible.
* State remains deterministic.
* EVM remains compatible.
* Governance remains decentralized.

End of Volume 3.
