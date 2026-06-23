# Economic Simulation: Decentralized Exchange (Aruna Swap)

This document defines the transaction fee splits, liquidity provider (LP) rewards model, and yield simulations for the native automated market maker (AMM) DEX.

## 1. DEX Trading Fee Model
ARUNA Swap implements a standard Constant Product AMM ($x \cdot y = k$) with a flat transaction fee of **0.30%** on all swaps. The fee is deducted from the input asset and distributed as follows:

* **Liquidity Providers (70% of fee / 0.21% of volume):** Added back to the pool to increase the value of LP shares.
* **Treasury (20% of fee / 0.06% of volume):** Transferred to the Treasury account.
* **Protocol Reserve (10% of fee / 0.03% of volume):** Allocated to node validators.

---

## 2. Liquidity Yield Simulation
We simulate the returns for Liquidity Providers in an `ARU` / `ARC-20 Token` pool:

* **Simulation Parameters:**
  * **Total Pool Liquidity:** $100,000 \text{ ARU}$ value.
  * **Average Daily Trade Volume:** $10,000 \text{ ARU}$.

### Step-by-Step Fee Calculations:
1. **Total Daily Fees Generated:**
   $$\text{DailyFees} = 10,000 \text{ ARU} \times 0.30\% = 30 \text{ ARU}$$
2. **LP Reward Share (70%):**
   $$\text{LPReward} = 30 \text{ ARU} \times 70\% = 21 \text{ ARU per day}$$
3. **Treasury Inflow (20%):**
   $$\text{TreasuryInflow} = 30 \text{ ARU} \times 20\% = 6 \text{ ARU per day}$$
4. **Protocol Reserve (10%):**
   $$\text{ProtocolReserve} = 30 \text{ ARU} \times 10\% = 3 \text{ ARU per day}$$

### Annualized Yield (APY) for LPs:
$$\text{LP\_APY} = \frac{21 \text{ ARU} \times 365 \text{ days}}{100,000 \text{ ARU}} \times 100\% = 7.665\% \text{ APY}$$
* **Economic Analysis:** A 7.66% APY provides a sustainable yield for liquidity providers without causing token inflation, encouraging participants to lock capital in native pools to earn trading fees.
