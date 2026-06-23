# ADR-0007: Maximum Supply 1 Billion ARU

## Status
Proposed

## Context
The monetary policy of a blockchain determines its long-term viability, economic sustainability, and credibility within the community. A core component of monetary policy is the token supply limit. There are two primary paradigms:
1. **Uncapped Supply (Inflationary):** E.g., Ethereum or Dogecoin. The supply grows indefinitely, which requires constant demand to maintain value but ensures perpetual block rewards.
2. **Capped Supply (Disinflationary):** E.g., Bitcoin or Litecoin. The total supply is strictly limited by code, introducing hard scarcity and protecting token holders from dilution.

ARUNA Network aims to build a community-owned payment and utility network that resists hyperinflation and founder dilution.

## Problem
Allowing an adjustable or uncapped supply introduces the following risks for ARUNA:
1. **Loss of Scarcity:** Infinite supply or hidden mint functions destroy the token's credibility as a store of value, discouraging long-term holders.
2. **Founder/Governance Abuse:** If the supply can be increased via governance or administrative bypasses, developers or large validators may mint new tokens, diluting the community.
3. **Economic Instability:** Unpredictable token generation rates lead to inflation, making transaction fee calculations and economic planning difficult.

We need an immutable supply cap that is hardcoded into the consensus engine and protected from any modifications.

## Decision
We enforce a strict **Maximum Supply of 1,000,000,000 ARU (One Billion ARU)**.

### Allocation Strategy:
The total supply is distributed as follows to prioritize mining and staking over centralized reserves:
* **Mining & Staking Ecosystem (92.0% - 920,000,000 ARU):** Distributed to miners (70%) and stakers (25%) over approximately 20+ years.
* **Founder Allocation (1.5% - 15,000,000 ARU):** Subject to a 48-month monthly linear vesting schedule with zero early unlock or governance privileges.
* **Premine Reserve (1.5% - 15,000,000 ARU):** Publicly auditable and reserved for testnet rewards, bootstrapper nodes, and security audits.
* **Treasury Reserve (5.0% - 50,000,000 ARU):** Reserved for ecosystem development, explorer nodes, and public infrastructures.

### Protocol Constraints:
* The consensus engine must block any state transition that attempts to mint supply exceeding this cap.
* **Supply cap immutability:** The maximum supply is an immutable rule of the ARUNA protocol and cannot be modified by governance or code upgrades.

## Alternatives
* **Alternative A: Capped Supply with Fee Burning (Ethereum-style EIP-1559):** Rejected because fee burning complicates reward calculation for low-power miners and reduces the long-term predictability of validator returns on low-cost hardware.
* **Alternative B: Capped Supply with dynamic emission adjustment:** Rejected because it introduces economic complexity and potential attack surfaces for governance capture.

## Consequences
* **Positive:**
  * **Absolute Scarcity:** Enforces hard scarcity, establishing ARU as a long-term store of value.
  * **Economic Credibility:** A clear, predictable emission schedule encourages early mining participation.
  * **Security Against Abuse:** The omission of emergency mint or admin functions prevents key compromise exploits.
* **Negative:**
  * Once the maximum supply is approached, block rewards will decrease significantly. The network must transition to a transaction-fee-driven economy to sustain validators and miners.

## Migration
Not applicable. The maximum supply cap is defined in the genesis block parameter settings.

## Security Impact
No admin key, developer key, or validator majority can modify this parameter. The maximum supply is checked during block validation by every node in the P2P network. Any block containing transactions that attempt to mint new tokens outside the defined emission schedule is immediately rejected as invalid.
