# ARUNA NETWORK

## MASTER PRD

### Volume 8 — Security Architecture & Threat Model

---

# SECURITY PHILOSOPHY

ARUNA adopts a **Zero Trust Security Model** under these assumptions:
* Every peer can be malicious.
* Every validator can be malicious.
* Every miner can be malicious.
* Every wallet can be compromised.
* Every transaction can be adversarial.

*The protocol must remain secure even when participants act against network interests.*

---

# SECURITY PRIORITY ORDER

Security constraints follow a strict hierarchy:
```
Consensus Safety → Fund Safety → State Integrity → Network Availability → Performance
```
* **Rule:** Security always overrides convenience.

---

# CONSENSUS THREAT MODEL & MITIGATIONS

### 51% Attack (Majority mining power)
* **Risk:** Double spend, chain reorganization, and transaction censorship.
* **Mitigation:** Hybrid consensus. Chain selection requires both **PoW Cumulative Work Weight** + **PoS Finalized Stake Weight**. Mining power alone cannot reorganize the chain.

### Long Range Attack (Old validator keys rewriting history)
* **Risk:** Historical state reorganization.
* **Mitigation:** Checkpoint finality. Validators sign and finalize blocks at epoch boundaries. Finalized checkpoints become immutable.

### Validator Collusion
* **Risk:** Vote manipulation, governance capture, and treasury abuse.
* **Mitigation:** Automatic monitoring of stake distribution, delegation diversity, and open, transparent governance.

---

# NETWORK & TRANSACTION SECURITY

### Replay Attack Defense
* **Mitigation:** Every transaction must include a unique `Chain ID` and `Network ID` (e.g. Sumatera Testnet transactions are rejected on Jawa Mainnet).
* **Double Spend Protection:** Strict nonce sequencing, local signature validation, and state validation.

### Eclipse Attack (Attacker isolates a node)
* **Mitigation:** Minimum 16 independent connections (recommended 32+), using geographically diverse peers from different ASNs.

### DDoS & Mempool Spam Attack
* **Mitigation:** Node rate-limiting, connection limits, validation checks before relaying messages, transaction fee floors, and mempool prioritizations.
* **RPC Protection:** Mandatory API rate-limiting on RPC nodes.
* **Relaying Rules:** Never relay unvalidated blocks, transactions, or metadata.

---

# CRYPTOGRAPHIC SECURITY

* **Approved Algorithms:** Ed25519, secp256k1, BLAKE3, Argon2, AES.
* **Forbidden Primitives:** Custom encryption, custom signatures, unreviewed cryptographic libraries.

---

# WALLET SECURITY

* **Keys Protection:** Biometric unlock, PIN protection, local signing via Android Keystore/Secure Enclave. Keys never leave the local device.
* **Seed Security:** Standard BIP39 seed phrases. Wallets must never upload, transmit, or log seed phrases.
* **Phishing Defense:** Address checksums, verification prompts, transaction previews, and compiler/suspicious contract warnings.

---

# SMART CONTRACT & EVM SECURITY

* **Resource Limits:** Strict gas limits, memory limits, execution limits, and storage limits to prevent state exhaustion and infinite execution DoS.
* **Ecosystem Protections:** Suspicious contracts flagged in explorer/wallet. OpenZeppelin patterns recommended.
* **Execution Constraints:** Smart contracts cannot override consensus rules or treasury controllers.

---

# GOVERNANCE & TREASURY SECURITY

* **No Superuser:** The founder possesses no protocol backdoor, override privileges, emergency mint functions, or hidden controls. The network must survive founder absence.
* **Treasury Security:** The treasury must never be controlled by a founder wallet, single validator, or static MultiSig. All treasury movements are controlled by on-chain governance contracts.
* **Governance Capture Defense:** Hybrid governance requiring voting periods, review periods, and execution time-delays for treasury disbursements.

---

# CENTRALIZATION MONITORING

* **Mining Centralization:** Alert generated if any single pool controls **>25%** hashrate; critical threshold at **>40%**.
* **Validator Centralization:** Automate staking concentration reports for the Top 10 and Top 25 validators.

---

# INCIDENT RESPONSE & AUDIT FRAMEWORKS

### Incident Severity Levels:
* **Level 1:** UI Bug
* **Level 2:** Node Bug
* **Level 3:** Consensus Bug
* **Level 4:** Critical Network Threat
* **Level 5:** Chain Integrity Risk
* *Response requires disclosure, investigation, patching, review, and public postmortem.*

### Audits & Bounties:
* Staking, consensus, networking, DEX, and governance must undergo professional security audits.
* Continuous bug bounty program funded by the Treasury.

---

# SUCCESS CRITERIA

Security succeeds when:
1. Consensus remains safe under adversarial conditions.
2. User funds and the treasury remain protected.
3. Staking and mining remain decentralized.
4. Android devices operate safely.
5. Founder key compromise does not compromise the chain.

End of Volume 8.
