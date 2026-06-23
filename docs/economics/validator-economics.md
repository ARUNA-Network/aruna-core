# Economic Simulation: Validator Economics (PoS APR)

This document calculates the Proof of Stake (PoS) yields and Annual Percentage Rate (APR) dynamics for ARUNA validators and delegators.

## 1. Staking Rewards Base
* **PoS Reward Per Block:** 6.25 ARU.
* **Blocks Per Year:** 1,051,200 blocks.
* **Total PoS Inflation Emitted Per Year:**
  $$\text{AnnualPoSReward} = 1,051,200 \text{ blocks} \times 6.25 \text{ ARU} = 6,570,000 \text{ ARU}$$
* **Validator Minimum Stake:** 10,000 ARU.

Staking rewards are distributed proportionally based on stake weight. The APR formula is:
$$\text{StakingAPR} = \frac{\text{AnnualPoSReward}}{\text{TotalStakedWeight}} \times 100\%$$

---

## 2. Validator Yield Simulations

### Scenario A: 100 Validators (Min Stake Only)
Assume 100 active validator nodes, each operating with only the minimum self-stake of 10,000 ARU.
* **Total Staked Weight:**
  $$100 \text{ validators} \times 10,000 \text{ ARU} = 1,000,000 \text{ ARU}$$
* **APR Calculation:**
  $$\text{APR} = \frac{6,570,000 \text{ ARU}}{1,000,000 \text{ ARU}} \times 100\% = 657.0\% \text{ APR}$$
* **Economic Analysis:** This high APR occurs because only 1M ARU is locked out of the Year 1 circulating supply (~45M ARU). It provides an extreme incentive to boot up validation infrastructure in the early testnet stages.

### Scenario B: 100 Validators (Staked + Delegated)
Assume 100 validators, but community delegators stake additional coins, raising the average stake per validator to 50,000 ARU.
* **Total Staked Weight:**
  $$100 \text{ validators} \times 50,000 \text{ ARU} = 5,000,000 \text{ ARU}$$
* **APR Calculation:**
  $$\text{APR} = \frac{6,570,000 \text{ ARU}}{5,000,000 \text{ ARU}} \times 100\% = 131.4\% \text{ APR}$$

---

### Scenario C: 1,000 Validators (Min Stake Only)
Assume the network has grown to 1,000 active validator nodes, each operating at the minimum self-stake of 10,000 ARU.
* **Total Staked Weight:**
  $$1,000 \text{ validators} \times 10,000 \text{ ARU} = 10,000,000 \text{ ARU}$$
* **APR Calculation:**
  $$\text{APR} = \frac{6,570,000 \text{ ARU}}{10,000,000 \text{ ARU}} \times 100\% = 65.7\% \text{ APR}$$

### Scenario D: 1,000 Validators (Staked + Delegated)
Assume 1,000 validators with delegator participation, raising the average stake per validator to 30,000 ARU.
* **Total Staked Weight:**
  $$1,000 \text{ validators} \times 30,000 \text{ ARU} = 30,000,000 \text{ ARU}$$
* **APR Calculation:**
  $$\text{APR} = \frac{6,570,000 \text{ ARU}}{30,000,000 \text{ ARU}} \times 100\% = 21.9\% \text{ APR}$$
* **Economic Analysis:** As the total stake weight grows (reaching ~22% of the Year 4 circulating supply), the yield stabilizes at ~21.9% APR. This remains highly competitive compared to traditional assets while keeping coin velocity healthy.
