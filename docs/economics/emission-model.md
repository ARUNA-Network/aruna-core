# Economic Simulation: Emission Model

This document models the token emission, vesting schedule, and circulating supply metrics for the ARUNA Network over a 5-year timeline.

## 1. Emission Parameters
* **Block Time:** 30 seconds.
* **Blocks Per Day:** 2,880 blocks.
* **Blocks Per Year (365 days):** 1,051,200 blocks.
* **Block Reward (Era 1):** 25 ARU.
* **Maximum Supply Cap:** 1,000,000,000 ARU (Immutable).

## 2. Supply Components

### A. Premine (Genesis Release)
* **Amount:** 15,000,000 ARU (1.5% of total supply).
* **Circulation Status:** 100% circulating at genesis for bootstrapper nodes, liquidity bootstrapping, and security audits.

### B. Founder Vesting
* **Amount:** 15,000,000 ARU (1.5% of total supply).
* **Vesting Schedule:** 48 Months linear monthly release.
* **Monthly Release Rate:** $15,000,000 / 48 = 312,500 \text{ ARU per month}$.

### C. Block Rewards
* **Era 1 emission:** 25 ARU per block (cuts to 12.5 ARU at block 4,204,800).

---

## 3. Circulating Supply Simulations

### Simulation A: Year 1 Circulating Supply
Calculated at block height **1,051,200** (end of year 1):

1. **Block Rewards Minted:**
   $$1,051,200 \text{ blocks} \times 25 \text{ ARU} = 26,280,000 \text{ ARU}$$
2. **Premine in Circulation:**
   $$15,000,000 \text{ ARU}$$
3. **Vested Founder Allocation (12 months):**
   $$12 \text{ months} \times 312,500 \text{ ARU} = 3,750,000 \text{ ARU}$$
4. **Total Year 1 Circulating Supply:**
   $$\text{Block Rewards} + \text{Premine} + \text{Vested Founder} = 45,030,000 \text{ ARU}$$

### Simulation B: Year 4 (First Halving) Circulating Supply
Calculated at block height **4,204,800** (end of year 4/first halving block):

1. **Block Rewards Minted (Era 1):**
   $$4,204,800 \text{ blocks} \times 25 \text{ ARU} = 105,120,000 \text{ ARU}$$
2. **Premine in Circulation:**
   $$15,000,000 \text{ ARU}$$
3. **Vested Founder Allocation (48 months - Fully Vested):**
   $$48 \text{ months} \times 312,500 \text{ ARU} = 15,000,000 \text{ ARU}$$
4. **Total Year 4 Circulating Supply:**
   $$\text{Block Rewards} + \text{Premine} + \text{Vested Founder} = 135,120,000 \text{ ARU}$$

At this point, block rewards automatically halve to **12.5 ARU**, starting Era 2.
