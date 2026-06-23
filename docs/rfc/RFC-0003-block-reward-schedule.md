# RFC-0003: Block Reward Schedule & Supply Distribution

## Status
Accepted

## 1. Background
This RFC outlines the disinflationary emission schedule and genesis distribution details to enforce the 1,000,000,000 ARU total supply cap.

## 2. Proposed Specification
The block rewards start at 25 ARU and cut in half every 4,204,800 blocks (~4 years):

1. **Emission Splits (fixed at the protocol level):**
   * **PoW Miner:** 70% ($17.5 \text{ ARU}$)
   * **PoS Staker:** 25% ($6.25 \text{ ARU}$)
   * **Treasury:** 5% ($1.25 \text{ ARU}$)
2. **Reward Halving Decays (micro-ARU integer representation):**
   * **Era 1 (Blocks 0 - 4,204,799):** 25,000,000 $\mu\text{ARU}$
   * **Era 2 (Blocks 4,204,800 - 8,409,599):** 12,500,000 $\mu\text{ARU}$
   * **Era 3 (Blocks 8,409,600 - 12,614,399):** 6,250,000 $\mu\text{ARU}$
   * **Era 4 (Blocks 12,614,400 - 16,819,199):** 3,125,000 $\mu\text{ARU}$
   * **Era 5 (Blocks 16,819,200 - 21,023,999):** 1,562,500 $\mu\text{ARU}$
3. **Genesis Allocations:**
   * **Mining & Staking Pool:** 920,000,000 ARU.
   * **Founder Vesting Pool:** 15,000,000 ARU (vested linearly over 48 months).
   * **Premine Reserve:** 15,000,000 ARU.
   * **Treasury Reserve:** 50,000,000 ARU.
