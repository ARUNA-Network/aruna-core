# Protocol Specification: Block Rewards & Distribution

This document defines the formulas, currency precision, halving decay calculations, and integer remainder management for block reward allocations.

## 1. Currency Precision: micro-ARU
To maintain absolute precision and prevent floating-point rounding errors that can cause consensus split vulnerabilities:
* **Base Unit:** ARU.
* **Minimal Unit:** micro-ARU ($\mu\text{ARU}$).
* **Conversion:** $1 \text{ ARU} = 1,000,000 \mu\text{ARU}$.
* **Data Type:** All currency balances are stored and computed as **64-bit unsigned integers (`u64`)**.

## 2. Block Reward Distribution Splits
Each block reward is split into three shares:
* **Proof of Work Miner Share (70%):** Awarded to the miner solving the AHash puzzle.
* **Proof of Stake Validator Share (25%):** Distributed among validators and delegators based on staked weight.
* **Treasury Share (5%):** Routed to the governance-controlled network treasury account.

---

## 3. Halving Schedule & Reward Decay (in micro-ARU)
The block reward decays at 4,204,800 block intervals (Eras). All payouts are represented as fixed integer values in micro-ARU:

| Era | Total Reward ($\mu\text{ARU}$) | Miner Share (70%) | Validator Share (25%) | Treasury Share (5%) |
| :--- | :--- | :--- | :--- | :--- |
| **Era 1** | $25,000,000$ | $17,500,000$ | $6,250,000$ | $1,250,000$ |
| **Era 2** | $12,500,000$ | $8,750,000$ | $3,125,000$ | $625,000$ |
| **Era 3** | $6,250,000$ | $4,375,000$ | $1,562,000$ | $312,500$ + *dust* |
| **Era 4** | $3,125,000$ | $2,187,500$ | $781,250$ | $156,250$ |
| **Era 5** | $1,562,500$ | $1,093,750$ | $390,625$ | $78,125$ |

---

## 4. Integer Division & Dust (Remainder) Management
Because block rewards are integers, dividing them by fractions can produce remainders (dust fractions of micro-ARU).
* **Formula:**
  $$\text{MinerShare} = \text{TotalReward} \times 70 / 100$$
  $$\text{ValidatorShare} = \text{TotalReward} \times 25 / 100$$
  $$\text{TreasuryShare} = \text{TotalReward} - (\text{MinerShare} + \text{ValidatorShare})$$
* **Dust Allocation:** By defining the `TreasuryShare` as the remainder of the total block reward minus the miner and validator shares, **all division remainders (dust) are automatically swept to the Treasury**. This prevents coins from being permanently locked or creating hidden inflation discrepancies.
