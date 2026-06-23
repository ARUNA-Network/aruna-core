# RFC-0010: Staking Parameters & Validator Penalty Rules

## Status
Accepted

## 1. Background
This RFC specifies the staking thresholds, delegator commissions, and validator offline penalty rules.

## 2. Proposed Specification

### A. Staking Thresholds & Commissions
* **Validator Minimum Stake:** **10,000 ARU** (10B micro-ARU).
* **Delegator Minimum Stake:** **100 ARU** (to prevent balance fragmentation).
* **Commission Rate:** Operator-defined, recommended between **0%–10%**.

### B. Validator Offline Penalty (No Slashing)
To support home lab and community node operators who may experience minor power or connectivity dropouts, ARUNA rejects punitive stake slashing:
1. **No Stake Burning:** Staked capital is never destroyed or burned due to node downtime.
2. **Reward Loss:** An offline validator node is immediately excluded from the block signature selection loop, losing **100% of staking rewards** for the duration of its offline state.
3. **Heartbeat Recovery:** Validators can return to active status by broadcasting a signed on-chain heartbeat transaction once back online.
