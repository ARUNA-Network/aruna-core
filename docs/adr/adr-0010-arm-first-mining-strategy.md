# ADR-0010: ARM First Mining Strategy

## Status
Proposed

## Context
ARUNA's core mission is to enable decentralized blockchain mining on consumer hardware. The most abundant computing devices globally are ARM-based processors, which power billions of Android smartphones, smart TVs, Raspberry Pi boards, and single-board computers.

However, consumer devices have strict physical constraints that do not apply to server rooms or dedicated mining setups:
1. **Thermal Dissipation:** Mobile phones and single-board computers are passively cooled. Continuous 100% CPU utilization will cause thermal throttling, battery degradation, and potential hardware damage.
2. **Battery Life:** Mining on mobile batteries will rapidly drain and degrade the cell.
3. **Bandwidth Limitations:** Mobile devices operate on cellular data (4G/5G) or shared home Wi-Fi networks with limited bandwidth and intermittent connectivity.

## Problem
If the mining protocol treats ARM-based consumer devices the same as x86 servers, several issues arise:
1. **Hardware Damage:** Android devices could experience battery swelling or hardware failures due to continuous high thermal loads, destroying the project's reputation and trust.
2. **Poor User Experience:** If mining renders the phone laggy or rapidly depletes the battery, users will uninstall the miner app.
3. **Data Overhead:** Syncing a full blockchain over mobile networks will consume users' cellular data caps, creating high costs.

We need an execution strategy that optimizes for ARM processor constraints, monitors device safety, and operates seamlessly in the background.

## Decision
We enforce an **ARM First Mining Strategy** with strict device-level rules in our client applications (specifically `apps/miner-mobile` and the underlying mining protocol):

### 1. Thermal Management & Throttling
* The miner app must continuously monitor CPU and battery temperature sensors.
* **Throttling Threshold (e.g., CPU > 65°C or Battery > 40°C):** The miner must automatically reduce the number of active threads or introduce sleep cycles (cooling cycles) to lower the hashrate.
* **Critical Threshold (e.g., CPU > 75°C or Battery > 45°C):** The miner must immediately pause mining and notify the user.

### 2. Battery Protection Rules
* **Battery Level Threshold:** Mining must stop automatically if the device battery is below 20% (unless explicitly overridden by the user).
* **Charging Toggles:** The default setting must require the device to be plugged into a power source (charging) for mining to execute.
* **Auto-Resume:** Mining must automatically resume once the device begins charging and the battery exceeds the safe threshold.

### 3. Screen-Off & Background Execution
* The mobile miner (`Aruna Mine`) must support background screen-off mining. It must run as a persistent background service (e.g., using Android Foreground Services with a visible notification) to prevent the OS from killing the process.
* The mining thread priority must be set to `background/idle` to ensure the device remains responsive for everyday phone operations.

### 4. Bandwidth Optimization (Light Miner)
* Mobile miners operate as **Light Nodes**. They do not download or store the full blockchain database.
* The P2P protocol must support a **Header-Only Work Distribution**: the node sends block headers and difficulty targets, and the mobile miner returns the solved block header fields (nonces) once found. This reduces bandwidth usage to a few megabytes per day.

## Alternatives
* **Alternative A: Cloud/Server-Only Mining:** Prioritize x86 servers and high-end VMs. This was rejected because it violates the accessibility and decentralization principles, excluding the Indonesian community.
* **Alternative B: Web-Browser Mining (JS-based):** Easier to deploy, but lacks access to hardware-accelerated NEON/AES instructions, resulting in highly inefficient, battery-draining execution.

## Consequences
* **Positive:**
  * **Safe Operation:** Guarantees that mining will not damage users' mobile hardware.
  * **Ecosystem Growth:** Lowers the entry barrier, allowing millions of smartphone owners to participate.
  * **Background Viability:** Mining runs silently in the background while charging overnight, converting idle electricity and hardware into network security.
* **Negative:**
  * Hashrates on mobile devices will be lower than on active, actively cooled x86 machines. However, this is offset by the massive number of mobile devices participating.

## Migration
Not applicable. These constraints will be coded directly into the mobile miner client and validated against the P2P headers protocol.

## Security Impact
By supporting millions of independent mobile miners, the network becomes highly resilient to physical location attacks, cloud provider outages, and regulatory actions. The decentralized hashrate distribution prevents pool monopolies from easily capturing consensus.
