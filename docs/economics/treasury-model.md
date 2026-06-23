# Economic Simulation: Treasury Model & Sustainability

This document models the Treasury reserve inflows, growth trajectories, and spending models over a 5-year timeline.

## 1. Treasury Reserve Inflows
The ARUNA Treasury is designed to transition from an inflation-funded model to an ecosystem-revenue-driven model:

1. **Fixed Block Rewards (5%):**
   * Era 1: 1.25 ARU per block.
   * **Annual Inflow:** $1,051,200 \text{ blocks} \times 1.25 \text{ ARU} = 1,314,000 \text{ ARU per year}$.
2. **Initial Reserve Allocation:**
   * **Genesis Balance:** 50,000,000 ARU (5% of the 1B total supply).
3. **Ecosystem Supplemental Fees (Post-MVP):**
   * **DEX Swaps:** 20% of the 0.30% native swap fee (equivalent to 0.06% of total trade volume).
   * **Domain Registrations:** 100% of ARU domain fees (e.g. `example.aruna`).
   * **Launchpad & Bridge:** Listing and bridging platform fees.

---

## 2. 5-Year Cumulative Treasury Inflow Model
We simulate the growth of the Treasury, assuming a baseline network growth and transaction fee activity:

* **Baseline Assumptions:**
  * Year 1–3: Minimal transaction fees (testnets Sumatera, Kalimantan, Sulawesi).
  * Year 4: Papua RC launch with EVM/DEX. DEX volume averages 50,000 ARU/day. Domain registrations average 1,000 domains/year at 10 ARU/domain.
  * Year 5: Jawa Mainnet launch. DEX volume averages 500,000 ARU/day. Domain registrations average 5,000 domains/year.

### Inflow Trajectory Matrix (ARU):
* **Year 1:**
  * *Inflows:* 1,314,000 (Block rewards) + 50,000,000 (Genesis Reserve).
  * *Cumulative Balance:* **51,314,000 ARU**.
* **Year 2:**
  * *Inflows:* 1,314,000 (Block rewards).
  * *Cumulative Balance:* **52,628,000 ARU**.
* **Year 3:**
  * *Inflows:* 1,314,000 (Block rewards).
  * *Cumulative Balance:* **53,942,000 ARU**.
* **Year 4 (Halving occurs mid-year - block reward drops to 0.625 ARU):**
  * *Inflows:* 985,500 (Block rewards) + 10,950 (DEX fee cuts) + 10,000 (Domain fees).
  * *Cumulative Balance:* **54,948,450 ARU**.
* **Year 5 (Mainnet):**
  * *Inflows:* 657,000 (Block rewards) + 109,500 (DEX fee cuts) + 50,000 (Domain fees).
  * *Cumulative Balance:* **55,764,950 ARU**.

---

## 3. Long-Term Sustainability Analysis
As shown in Year 5, block reward emissions decrease due to halvings, but ecosystem fees (DEX volume and naming services) scale up. This fee replacement ensures that the Treasury remains solvent and capable of funding continuous open-source grants and security audits indefinitely, without relying on infinite token minting.
