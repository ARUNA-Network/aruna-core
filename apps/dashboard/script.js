// ARUNA Network Dashboard - script.js
import { blake3 } from 'https://esm.sh/@noble/hashes/blake3';
import { ed25519 } from 'https://esm.sh/@noble/curves/ed25519';

// Configuration
const NODE_RPC_URL = 'http://127.0.0.1:8080';
const HRP_PREFIX = 'sum';

// Global Wallet State
let currentPrivateKey = null; // Uint8Array (32 bytes)
let currentPublicKey = null;  // Uint8Array (32 bytes)
let currentAddress = null;    // String
let activeTab = 'explorer';
let pollIntervalId = null;

// Bech32m Constants & Logic
const CHARSET = 'qpzry9x8gf2tvdw0s3jn54khce6mua7l';
const BECH32M_CONST = 0x2bc830f3;
const GENERATOR = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];

function polymod(values) {
  let chk = 1;
  for (let i = 0; i < values.length; i++) {
    let top = chk >> 25;
    chk = ((chk & 0x1ffffff) << 5) ^ values[i];
    for (let j = 0; j < 5; j++) {
      if ((top >> j) & 1) {
        chk ^= GENERATOR[j];
      }
    }
  }
  return chk;
}

function hrpExpand(hrp) {
  const ret = [];
  for (let i = 0; i < hrp.length; i++) {
    ret.push(hrp.charCodeAt(i) >> 5);
  }
  ret.push(0);
  for (let i = 0; i < hrp.length; i++) {
    ret.push(hrp.charCodeAt(i) & 31);
  }
  return ret;
}

function verifyChecksum(hrp, data) {
  const combined = hrpExpand(hrp).concat(data);
  return polymod(combined) === BECH32M_CONST;
}

function createChecksum(hrp, data) {
  const combined = hrpExpand(hrp).concat(data).concat([0, 0, 0, 0, 0, 0]);
  const mod = polymod(combined) ^ BECH32M_CONST;
  const ret = [];
  for (let i = 0; i < 6; i++) {
    ret.push((mod >> (5 * (5 - i))) & 31);
  }
  return ret;
}

function convertBits(data, frombits, tobits, pad) {
  let acc = 0;
  let bits = 0;
  const ret = [];
  const maxv = (1 << tobits) - 1;
  const max_acc = (1 << (frombits + tobits - 1)) - 1;
  for (let i = 0; i < data.length; i++) {
    const value = data[i];
    acc = ((acc << frombits) | value) & max_acc;
    bits += frombits;
    while (bits >= tobits) {
      bits -= tobits;
      ret.push((acc >> bits) & maxv);
    }
  }
  if (pad) {
    if (bits > 0) {
      ret.push((acc << (tobits - bits)) & maxv);
    }
  } else if (bits >= frombits || ((acc << (tobits - bits)) & maxv) !== 0) {
    throw new Error('Invalid padding');
  }
  return ret;
}

// Encode binary data to Bech32m string
function encodeBech32m(hrp, data) {
  const converted = convertBits(data, 8, 5, true);
  const checksum = createChecksum(hrp, converted);
  const combined = converted.concat(checksum);
  let ret = hrp + '1';
  for (let i = 0; i < combined.length; i++) {
    ret += CHARSET[combined[i]];
  }
  return ret;
}

// Decode Bech32m string back to HRP and binary data
function decodeBech32m(s) {
  if (s.length < 8 || s.length > 90) throw new Error('Invalid address length');
  const lowercase = s.toLowerCase();
  const pos = lowercase.lastIndexOf('1');
  if (pos === -1 || pos === 0) throw new Error('Missing separator');
  const hrp = lowercase.slice(0, pos);
  const dataChars = lowercase.slice(pos + 1);
  if (dataChars.length < 6) throw new Error('Invalid checksum length');

  const data = [];
  for (let i = 0; i < dataChars.length; i++) {
    const idx = CHARSET.indexOf(dataChars[i]);
    if (idx === -1) throw new Error('Invalid character in address');
    data.push(idx);
  }

  if (!verifyChecksum(hrp, data)) throw new Error('Checksum validation failed');
  
  const payload = data.slice(0, -6);
  const decoded = convertBits(payload, 5, 8, false);
  return { hrp, data: decoded };
}

// Helper to convert Address (32 bytes, 12 padded + 20 pkh) to Bech32m sum1 Address
function addressBytesToBech32m(bytes) {
  const pkh = bytes.slice(12, 32);
  return encodeBech32m(HRP_PREFIX, Array.from(pkh));
}

// Helper to decode sum1 address back to 32 bytes Address array
function bech32mToAddressBytes(s) {
  const { hrp, data } = decodeBech32m(s);
  if (data.length !== 20) throw new Error('Invalid public key hash size');
  const addressBytes = new Uint8Array(32);
  addressBytes.set(data, 12);
  return addressBytes;
}

// Bincode Serialization for TransactionPayload (little-endian)
function serializeTransactionPayload(payload) {
  // Size: 8 (nonce) + 32 (sender) + 32 (recipient) + 8 (amount) + 8 (fee) + 8 (gas_limit) + 8 (gas_price) + 8 (data len) + data.length
  const dataLen = payload.data ? payload.data.length : 0;
  const bufferSize = 112 + dataLen;
  const buffer = new ArrayBuffer(bufferSize);
  const view = new DataView(buffer);
  
  // 1. Nonce (u64, LE)
  view.setBigUint64(0, BigInt(payload.nonce), true);
  
  // 2. Sender (32 bytes)
  const senderBytes = bech32mToAddressBytes(payload.sender);
  const u8Array = new Uint8Array(buffer);
  u8Array.set(senderBytes, 8);
  
  // 3. Recipient (32 bytes)
  const recipientBytes = bech32mToAddressBytes(payload.recipient);
  u8Array.set(recipientBytes, 40);
  
  // 4. Amount (u64, LE)
  view.setBigUint64(72, BigInt(payload.amount), true);
  
  // 5. Fee (u64, LE)
  view.setBigUint64(80, BigInt(payload.fee), true);
  
  // 6. Gas Limit (u64, LE)
  view.setBigUint64(88, BigInt(payload.gas_limit || 0), true);
  
  // 7. Gas Price (u64, LE)
  view.setBigUint64(96, BigInt(payload.gas_price || 0), true);
  
  // 8. Data Length (u64, LE)
  view.setBigUint64(104, BigInt(dataLen), true);
  
  // 9. Data bytes
  if (dataLen > 0) {
    u8Array.set(payload.data, 112);
  }
  
  return u8Array;
}

// UI Terminal log helper
function logToTerminal(message, type = 'system') {
  const terminal = document.getElementById('terminal-output');
  if (!terminal) return;
  const timestamp = new Date().toLocaleTimeString();
  let prefix = '[System]';
  if (type === 'error') prefix = '[ERROR]';
  if (type === 'success') prefix = '[OK]';
  if (type === 'tx') prefix = '[TX]';
  
  terminal.innerHTML += `\n${timestamp} ${prefix} ${message}`;
  terminal.scrollTop = terminal.scrollHeight;
}

// Fetch helper with timeout
async function fetchWithTimeout(url, options = {}, timeout = 5000) {
  const controller = new AbortController();
  const id = setTimeout(() => controller.abort(), timeout);
  try {
    const response = await fetch(url, {
      ...options,
      signal: controller.signal
    });
    clearTimeout(id);
    return response;
  } catch (e) {
    clearTimeout(id);
    throw e;
  }
}

// Query Node Status & update metrics
async function updateMetrics() {
  const connDot = document.querySelector('.status-dot');
  const connText = document.querySelector('.status-text');
  
  try {
    const res = await fetchWithTimeout(`${NODE_RPC_URL}/status`);
    if (!res.ok) throw new Error('Status not OK');
    const status = await res.json();
    
    // Update connection indicator
    connDot.className = 'status-dot connected';
    connText.textContent = 'Connected';
    
    // Update Metrics Dashboard
    document.getElementById('metric-height').textContent = status.height;
    
    // Get Tip Hash
    const tipRes = await fetch(`${NODE_RPC_URL}/chain/tip`);
    if (tipRes.ok) {
      const tip = await tipRes.json();
      document.getElementById('metric-tip').textContent = tip.hash.slice(0, 16) + '...';
      document.getElementById('metric-tip').title = tip.hash;
    }
    
    // Update mempool metric
    //Axum doesn't have direct /mempool_size but we can get it from logs or fallback
    document.getElementById('metric-mempool').textContent = 'Active';

    // If we have blocks tab open, refresh blocks table
    if (activeTab === 'explorer') {
      await refreshBlocksTable(status.height);
    }
    
  } catch (e) {
    connDot.className = 'status-dot disconnected';
    connText.textContent = 'Disconnected';
    document.getElementById('metric-height').textContent = '—';
    document.getElementById('metric-tip').textContent = '—';
    document.getElementById('metric-mempool').textContent = '—';
  }
}

// Load recent blocks into Explorer table
async function refreshBlocksTable(height) {
  const tbody = document.getElementById('blocks-tbody');
  if (!tbody) return;

  if (height === 0 || isNaN(height)) {
    tbody.innerHTML = `<tr><td colspan="6" style="text-align: center;">Genesis block committed. No secondary blocks produced yet.</td></tr>`;
    return;
  }

  try {
    let html = '';
    // Display last 10 blocks (from height down to max(1, height-9))
    const start = height;
    const end = Math.max(1, height - 9);
    
    for (let h = start; h >= end; h--) {
      const res = await fetch(`${NODE_RPC_URL}/block/${h}`);
      if (res.ok) {
        const block = await res.json();
        const shortHash = block.hash.slice(0, 16) + '...';
        const shortState = block.header.state_root.slice(0, 16) + '...';
        const shortMerkle = block.header.merkle_root.slice(0, 16) + '...';
        const timeStr = new Date(block.header.timestamp * 1000).toLocaleString();
        const txCount = block.body.transactions.length;
        
        html += `<tr onclick="inspectBlock(${h})">
          <td style="font-weight: 600; color: var(--accent-orange);">#${h}</td>
          <td class="code-font" title="${block.hash}">${shortHash}</td>
          <td class="code-font" title="${block.header.state_root}">${shortState}</td>
          <td class="code-font" title="${block.header.merkle_root}">${shortMerkle}</td>
          <td>${timeStr}</td>
          <td style="font-weight: 600;">${txCount}</td>
        </tr>`;
      }
    }
    tbody.innerHTML = html;
  } catch (e) {
    tbody.innerHTML = `<tr><td colspan="6" class="table-loading" style="color: var(--danger);">Failed to load recent blocks. Check RPC server.</td></tr>`;
  }
}

// Tab navigation handler
function setupTabs() {
  const navButtons = document.querySelectorAll('.nav-btn');
  const panes = document.querySelectorAll('.tab-pane');
  
  navButtons.forEach(btn => {
    btn.addEventListener('click', () => {
      const tab = btn.getAttribute('data-tab');
      activeTab = tab;
      
      navButtons.forEach(b => b.classList.remove('active'));
      panes.forEach(p => p.classList.remove('active'));
      
      btn.classList.add('active');
      document.getElementById(`tab-${tab}`).classList.add('active');
      
      logToTerminal(`Switched to ${tab.toUpperCase()} view.`);
      updateMetrics();
    });
  });
}

// Generate new random Ed25519 wallet
function generateNewWallet() {
  const seed = crypto.getRandomValues(new Uint8Array(32));
  loadWalletFromSeed(seed);
  logToTerminal('Generated new Ed25519 keys and derived Bech32m address.', 'success');
}

// Load keypair from seed bytes
function loadWalletFromSeed(seedBytes) {
  currentPrivateKey = seedBytes;
  currentPublicKey = ed25519.getPublicKey(seedBytes);
  
  // Compute address: BLAKE3(PubKey)[0..20] padded with 12 zeros
  const pubkeyHash = blake3(currentPublicKey).slice(0, 20);
  const addressBytes = new Uint8Array(32);
  addressBytes.set(pubkeyHash, 12);
  
  currentAddress = addressBytesToBech32m(addressBytes);
  
  // Update wallet setup UI
  document.getElementById('wallet-address').textContent = currentAddress;
  document.getElementById('wallet-pubkey').textContent = nobleHexEncode(currentPublicKey);
  document.getElementById('wallet-privkey').textContent = nobleHexEncode(currentPrivateKey);
  
  document.getElementById('keys-display-card').classList.remove('hidden');
  document.getElementById('btn-submit-tx').disabled = false;
  
  refreshWalletBalance();
}

// Import wallet from hex input
function importWallet() {
  const input = document.getElementById('wallet-import-seed').value.trim();
  if (input.length !== 64) {
    logToTerminal('Import failed: Hex seed must be exactly 64 characters (32 bytes).', 'error');
    alert('Invalid seed size. Must be exactly 32 hex bytes.');
    return;
  }
  try {
    const seedBytes = nobleHexDecode(input);
    loadWalletFromSeed(seedBytes);
    logToTerminal('Successfully imported Ed25519 keys from Hex seed.', 'success');
  } catch (e) {
    logToTerminal(`Import failed: ${e.message}`, 'error');
  }
}

// Query Address State from Node RPC
async function refreshWalletBalance() {
  if (!currentAddress) return;
  const balanceVal = document.getElementById('wallet-balance');
  const nonceVal = document.getElementById('wallet-nonce');
  const microVal = document.getElementById('wallet-microaru');
  
  balanceVal.textContent = 'loading...';
  
  try {
    const res = await fetch(`${NODE_RPC_URL}/address/${currentAddress}`);
    if (!res.ok) throw new Error('Address balance query failed');
    const state = await res.json();
    
    // Balance is returned in micro-ARU (1 ARU = 1,000,000 micro-ARU)
    const microAru = state.balance;
    const aru = (microAru / 1000000).toFixed(6);
    
    balanceVal.textContent = aru;
    nonceVal.textContent = state.nonce;
    microVal.textContent = microAru.toLocaleString();
    
    logToTerminal(`Wallet balance loaded: ${aru} ARU (Nonce: ${state.nonce}).`);
  } catch (e) {
    balanceVal.textContent = '0.000000';
    nonceVal.textContent = '0';
    microVal.textContent = '0';
    logToTerminal('Wallet not found on-chain. Requires pre-funding genesis allocations.', 'warning');
  }
}

// Submit a new transaction
async function signAndSubmitTransaction() {
  const to = document.getElementById('transfer-to').value.trim();
  const amountARU = parseFloat(document.getElementById('transfer-amount').value);
  const feeARU = parseFloat(document.getElementById('transfer-fee').value);
  
  if (!to || isNaN(amountARU) || isNaN(feeARU)) {
    alert('Please fill out all transaction fields.');
    return;
  }
  
  try {
    // 1. Fetch current nonce of sender
    const addressRes = await fetch(`${NODE_RPC_URL}/address/${currentAddress}`);
    let nonce = 0;
    if (addressRes.ok) {
      const state = await addressRes.json();
      nonce = state.nonce;
    }
    
    // Convert nonce for transaction (next nonce is nonce + 1)
    const txNonce = nonce + 1;
    const amountMicro = Math.round(amountARU * 1000000);
    const feeMicro = Math.round(feeARU * 1000000);
    
    // Validate inputs
    bech32mToAddressBytes(to); // throws if invalid Bech32m recipient
    
    const payload = {
      nonce: txNonce,
      sender: currentAddress,
      recipient: to,
      amount: amountMicro,
      fee: feeMicro,
      gas_limit: 0,
      gas_price: 0,
      data: []
    };
    
    logToTerminal(`Serializing transaction payload (LE Bincode layout, Nonce: ${txNonce})...`);
    const serializedBytes = serializeTransactionPayload(payload);
    
    logToTerminal('Signing payload bytes locally with Ed25519 private key...');
    const signature = ed25519.sign(serializedBytes, currentPrivateKey);
    
    const envelope = {
      payload,
      signature_type: 'Ed25519',
      signature: Array.from(signature),
      public_key: Array.from(currentPublicKey)
    };
    
    logToTerminal('Submitting TransactionEnvelope JSON to node RPC /tx...');
    const submitRes = await fetch(`${NODE_RPC_URL}/tx`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(envelope)
    });
    
    const response = await submitRes.json();
    if (submitRes.ok && response.status === 'success') {
      logToTerminal(`Transaction submitted! TX HASH: ${response.tx_hash}`, 'success');
      alert(`Success! Transaction submitted successfully.\nHash: ${response.tx_hash}`);
      
      // Auto-refresh wallet after submission
      setTimeout(refreshWalletBalance, 2000);
    } else {
      const err = response.message || JSON.stringify(response);
      logToTerminal(`Transaction submission rejected: ${err}`, 'error');
      alert(`Transaction Rejected:\n${err}`);
    }
    
  } catch (e) {
    logToTerminal(`Transaction execution failed: ${e.message}`, 'error');
    alert(`Error: ${e.message}`);
  }
}

// Explorer search logic
async function handleSearch() {
  const query = document.getElementById('search-input').value.trim();
  const errorDiv = document.getElementById('search-error');
  const resultCard = document.getElementById('search-result-card');
  const resultData = document.getElementById('result-data');
  
  errorDiv.textContent = '';
  if (!query) return;

  try {
    // 1. Is it a block height (number)?
    if (/^\d+$/.test(query)) {
      const res = await fetch(`${NODE_RPC_URL}/block/${query}`);
      if (!res.ok) throw new Error(`Block at height #${query} not found.`);
      const block = await res.json();
      displaySearchResult('Block Details', buildBlockHTML(block, query));
      return;
    }
    
    // 2. Is it a Bech32m wallet Address?
    if (query.startsWith('sum1') || query.startsWith('sumc1')) {
      const res = await fetch(`${NODE_RPC_URL}/address/${query}`);
      if (!res.ok) throw new Error(`Address state for ${query} not found.`);
      const state = await res.json();
      displaySearchResult('Address Ledger State', buildAddressHTML(state, query));
      return;
    }
    
    // 3. Is it a Tx Hash or Block Hash (64-byte Hex)?
    if (query.length === 64 && /^[0-9a-fA-F]+$/.test(query)) {
      // Try fetching as transaction first
      const txRes = await fetch(`${NODE_RPC_URL}/transaction/${query}`);
      if (txRes.ok) {
        const tx = await txRes.json();
        displaySearchResult('Transaction Details', buildTransactionHTML(tx));
        return;
      }
      
      // If not transaction, search block by hash? (Optional fallback - search through recent heights)
      throw new Error(`Hash ${query.slice(0, 16)}... not found as a transaction.`);
    }
    
    throw new Error('Unsupported search format. Enter height (number), address (sum1...), or hash (64 hex).');
    
  } catch (e) {
    errorDiv.textContent = e.message;
    resultCard.classList.add('hidden');
  }
}

// HTML Builders for Search Details
function buildBlockHTML(block, height) {
  const time = new Date(block.header.timestamp * 1000).toLocaleString();
  return `<div class="detail-row">
    <span class="detail-label">Height</span>
    <span class="detail-value" style="color: var(--accent-orange);">#${height}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Block Hash</span>
    <span class="detail-value code-font">${block.hash}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Previous Block Hash</span>
    <span class="detail-value code-font">${block.header.prev_block_hash}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">State Root</span>
    <span class="detail-value code-font">${block.header.state_root}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Merkle Root</span>
    <span class="detail-value code-font">${block.header.merkle_root}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Timestamp</span>
    <span class="detail-value">${time}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Difficulty</span>
    <span class="detail-value code-font">${block.header.difficulty.difficulty}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Nonce</span>
    <span class="detail-value code-font">${block.header.nonce}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Transactions Count</span>
    <span class="detail-value">${block.body.transactions.length}</span>
  </div>`;
}

function buildAddressHTML(state, address) {
  return `<div class="detail-row">
    <span class="detail-label">Address</span>
    <span class="detail-value code-fontHighlight">${address}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Balance</span>
    <span class="detail-value">${(state.balance / 1000000).toFixed(6)} ARU</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Micro-ARU Balance</span>
    <span class="detail-value">${state.balance.toLocaleString()}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Nonce</span>
    <span class="detail-value code-font">${state.nonce}</span>
  </div>`;
}

function buildTransactionHTML(res) {
  const tx = res.transaction;
  const senderAddr = addressBytesToBech32m(new Uint8Array(tx.payload.sender.pubkey_hash || tx.payload.sender.Address || tx.payload.sender));
  const recAddr = addressBytesToBech32m(new Uint8Array(tx.payload.recipient.pubkey_hash || tx.payload.recipient.Address || tx.payload.recipient));
  
  return `<div class="detail-row">
    <span class="detail-label">TX Hash</span>
    <span class="detail-value code-font">${res.hash}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Status</span>
    <span class="detail-value" style="color: var(--success); font-weight: 700;">${res.status}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Included in Block</span>
    <span class="detail-value" style="color: var(--accent-orange);">#${res.block_height}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Sender</span>
    <span class="detail-value code-font">${senderAddr}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Recipient</span>
    <span class="detail-value code-font">${recAddr}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Amount</span>
    <span class="detail-value" style="font-size: 16px;">${(tx.payload.amount / 1000000).toFixed(6)} ARU</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Fee</span>
    <span class="detail-value">${(tx.payload.fee / 1000000).toFixed(6)} ARU</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Nonce</span>
    <span class="detail-value code-font">${tx.payload.nonce.nonce}</span>
  </div>
  <div class="detail-row">
    <span class="detail-label">Signature Scheme</span>
    <span class="detail-value">${res.transaction.signature_type}</span>
  </div>`;
}

function displaySearchResult(title, html) {
  const resultCard = document.getElementById('search-result-card');
  const resultTitle = document.getElementById('result-title');
  const resultData = document.getElementById('result-data');
  
  resultTitle.textContent = title;
  resultData.innerHTML = html;
  resultCard.classList.remove('hidden');
}

// Inspect a block when clicking on a table row
window.inspectBlock = async function(height) {
  document.getElementById('search-input').value = height.toString();
  await handleSearch();
  // Scroll to search card
  document.getElementById('search-result-card').scrollIntoView({ behavior: 'smooth' });
};

// Hex Encoding helper
function nobleHexEncode(bytes) {
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

// Hex Decoding helper
function nobleHexDecode(hexStr) {
  if (hexStr.length % 2 !== 0) throw new Error('Hex string must have even length');
  const bytes = new Uint8Array(hexStr.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hexStr.substr(i * 2, 2), 16);
  }
  return bytes;
}

// Initialize listeners
function init() {
  setupTabs();
  
  // Search listeners
  document.getElementById('search-btn').addEventListener('click', handleSearch);
  document.getElementById('search-input').addEventListener('keypress', (e) => {
    if (e.key === 'Enter') handleSearch();
  });
  document.getElementById('close-result-btn').addEventListener('click', () => {
    document.getElementById('search-result-card').classList.add('hidden');
  });

  // Wallet Generation & Import
  document.getElementById('btn-generate-wallet').addEventListener('click', generateNewWallet);
  document.getElementById('btn-import-wallet').addEventListener('click', importWallet);
  document.getElementById('refresh-balance-btn').addEventListener('click', refreshWalletBalance);
  
  // Submit Tx listener
  document.getElementById('transfer-form').addEventListener('submit', signAndSubmitTransaction);

  // Clear Terminal console
  document.getElementById('clear-terminal-btn').addEventListener('click', () => {
    document.getElementById('terminal-output').textContent = '[Console cleared]';
  });

  // Refresh blocks list button
  document.getElementById('refresh-blocks-btn').addEventListener('click', () => {
    updateMetrics();
    logToTerminal('Refreshed blocks and network metrics from RPC.');
  });

  // Copy Buttons
  document.querySelectorAll('.copy-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      const targetId = btn.getAttribute('data-target');
      const text = document.getElementById(targetId).textContent;
      navigator.clipboard.writeText(text).then(() => {
        const originalText = btn.textContent;
        btn.textContent = 'Copied!';
        setTimeout(() => btn.textContent = originalText, 1500);
      });
    });
  });

  // Initial poll and update loops
  updateMetrics();
  pollIntervalId = setInterval(updateMetrics, 4000); // Poll every 4 seconds
}

// Start
document.addEventListener('DOMContentLoaded', init);
