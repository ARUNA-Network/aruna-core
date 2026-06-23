# Protocol Validation: Staking & Validator Specification

This document details the validator operations, minimum requirements, delegation rules, and penalty policies of the ARUNA Network.

## 1. Validator Minimum Requirements
To participate in block validation and cast votes on checkpoints, an account must register as a validator:
* **Minimum Stake Floor:** **10,000 ARU** (10,000,000,000 micro-ARU).
* **System Specs (Tier 2):** 4 CPU cores, 8 GB RAM, 100 GB SSD storage, and stable internet connectivity.
* **Validator Registration:** Executed by submitting a special protocol transaction (`validator_register`) containing the validator's public key, stake, and commission rate.

## 2. Validator Stake Limits & Cap
To maintain network decentralization and prevent validation monopoly:
* **Minimum Validator Stake:** 10,000 ARU.
* **Maximum Dominance Cap:** No single validator node may control **>10%** of the total network staked weight (including both self-stake and delegated stake).
* **Enforcement:** If a validator's stake exceeds the 10% limit, any delegated stake beyond 10% is excluded from voting weight calculations and rewards distribution, encouraging delegators to redirect their stake to smaller validators.

## 3. Delegated Staking & Commission
Users who do not run validator nodes can participate in staking by delegating their coins:
* **Staker Ownership:** Delegators retain ownership of their coins; delegated coins never leave the delegator's balance and cannot be spent by the validator.
* **Commission Rate:** Validators charge a fee on staking rewards. The recommended commission range is **0%–10%**, which is set by the validator operator and is visible on-chain.
* **Payouts:** Staking rewards are calculated on-chain and distributed automatically per block (6.25 ARU total split proportionally among stakers based on weight, minus validator commission).

## 4. Offline Policy (No Slashing)
ARUNA is built for community-operated nodes and home labs, which may experience unexpected downtime.
* **No Slashing Policy:** ARUNA does not implement punitive slashing. If a validator goes offline:
  1. No stake is burned or destroyed.
  2. The validator is temporarily flagged as inactive.
  3. The validator **loses all staking rewards** for the duration of the offline period.
  4. The validator can return to active status by syncing the chain and broadcasting a heartbeat transaction.
* **Security Rationale:** Punitive slashing (burning stake) penalizes ordinary community operators for home power/network failures, which centralizes validation into expensive cloud datacenters. Disabling rewards is a sufficient economic incentive to maintain uptime.
