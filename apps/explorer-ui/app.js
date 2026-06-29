/**
 * ARUNA Network Explorer — Frontend Application
 *
 * Architecture: reads exclusively from /api/v1/* REST endpoints.
 * No RocksDB, no Node RPC, no direct blockchain access.
 *
 * ADR-0017: Explorer Architecture
 */

const ARUNA = (() => {
  'use strict';

  // ── Configuration ────────────────────────────────────────────────────────────
  // Connect to the new TypeScript Worker API running on Cloudflare Edge by default
  const API_BASE = (window.ARUNA_API_URL || 'http://127.0.0.1:8787') + '/api/v1';
  const REFRESH_INTERVAL_MS = 12000;  // 12 seconds — slightly faster than block time
  const MICRO_ARU = 1_000_000;

  // ── API Client ───────────────────────────────────────────────────────────────
  async function apiFetch(path) {
    const res = await fetch(API_BASE + path, {
      headers: { 'Accept': 'application/json' }
    });
    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      throw new Error(body.error || `HTTP ${res.status}`);
    }
    return res.json();
  }

  const api = {
    stats:          ()          => apiFetch('/stats'),
    status:         ()          => apiFetch('/status'),
    blocks:         (l, o)     => apiFetch(`/blocks?limit=${l}&offset=${o}`),
    blockLatest:    ()          => apiFetch('/block/latest'),
    blockByHeight:  (n)         => apiFetch(`/block/height/${n}`),
    blockByHash:    (h)         => apiFetch(`/block/hash/${h}`),
    transaction:    (hash)      => apiFetch(`/transaction/${hash}`),
    address:        (addr, l, o) => apiFetch(`/address/${addr}?limit=${l}&offset=${o}`),
    search:         (q)         => apiFetch(`/search?q=${encodeURIComponent(q)}`),
    peers:          ()          => apiFetch('/peers').catch(() => ({ peers: [] })),
    validators:     ()          => apiFetch('/validators').catch(() => ({ active_validators_count: 1 })),
  };

  // ── Formatters ───────────────────────────────────────────────────────────────
  function shortHash(h = '') {
    if (!h || h.length < 16) return h;
    return h.slice(0, 8) + '…' + h.slice(-6);
  }

  function microAruToAru(micro) {
    return (Number(micro) / MICRO_ARU).toLocaleString('en-US', { maximumFractionDigits: 6 });
  }

  function timeAgo(unixSecs) {
    const diff = Math.floor(Date.now() / 1000) - Number(unixSecs);
    if (diff < 5)   return 'just now';
    if (diff < 60)  return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return new Date(Number(unixSecs) * 1000).toLocaleDateString();
  }

  function timestamp(unixSecs) {
    return new Date(Number(unixSecs) * 1000).toLocaleString();
  }

  function numFmt(n) {
    return Number(n).toLocaleString('en-US');
  }

  // ── Router / Navigation ──────────────────────────────────────────────────────
  function getParam(name) {
    return new URLSearchParams(window.location.search).get(name);
  }

  function navigate(url) {
    window.location.href = url;
  }

  function setupSearch(formId, inputId) {
    const form = document.getElementById(formId);
    if (!form) return;
    form.addEventListener('submit', async (e) => {
      e.preventDefault();
      const q = document.getElementById(inputId)?.value?.trim();
      if (!q) return;

      try {
        const results = await api.search(q);
        if (results.length === 0) {
          alert('No results found for: ' + q);
          return;
        }
        const first = results[0];
        if (first.kind === 'block') {
          navigate(`block.html?hash=${encodeURIComponent(first.value)}`);
        } else if (first.kind === 'transaction') {
          navigate(`tx.html?hash=${encodeURIComponent(first.value)}`);
        } else if (first.kind === 'address') {
          navigate(`address.html?addr=${encodeURIComponent(first.value)}`);
        }
      } catch (err) {
        if (/^\d+$/.test(q)) {
          navigate(`block.html?height=${encodeURIComponent(q)}`);
        } else if (q.length === 64) {
          navigate(`block.html?hash=${encodeURIComponent(q)}`);
        } else {
          navigate(`address.html?addr=${encodeURIComponent(q)}`);
        }
      }
    });
  }

  // ── DOM Helpers ──────────────────────────────────────────────────────────────
  function el(id) { return document.getElementById(id); }

  function setText(id, text) {
    const e = el(id);
    if (e) { e.textContent = text; e.classList.remove('skeleton'); }
  }

  function setHtml(id, html) {
    const e = el(id);
    if (e) e.innerHTML = html;
  }

  function showError(containerId, message) {
    setHtml(containerId, `
      <div class="error-state">
        <span class="error-icon">⚠️</span>
        <p>Failed to load data</p>
        <span class="error-msg">${escHtml(message)}</span>
      </div>
    `);
  }

  function escHtml(s) {
    return String(s)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  // ── Block List Item ──────────────────────────────────────────────────────────
  function blockListItem(b) {
    const txText = b.tx_count === 1 ? '1 tx' : `${b.tx_count} txs`;
    return `
      <a href="block.html?height=${b.height}" class="list-item block-row" role="listitem"
         aria-label="Block ${b.height}">
        <span class="block-height-badge">#${numFmt(b.height)}</span>
        <span class="hash-short">${escHtml(shortHash(b.hash))}</span>
        <span class="item-meta">${escHtml(timeAgo(b.timestamp))}</span>
        <span class="tx-count-badge">${txText}</span>
      </a>
    `;
  }

  // ── Transaction List Item ────────────────────────────────────────────────────
  function txListItem(tx) {
    return `
      <a href="tx.html?hash=${encodeURIComponent(tx.hash)}" class="list-item tx-row" role="listitem"
         aria-label="Transaction ${tx.hash}">
        <span class="hash-short">${escHtml(shortHash(tx.hash))}</span>
        <span class="item-meta">
          <a href="address.html?addr=${encodeURIComponent(tx.sender)}" onclick="event.stopPropagation()">${escHtml(shortHash(tx.sender))}</a>
          → <a href="address.html?addr=${encodeURIComponent(tx.recipient)}" onclick="event.stopPropagation()">${escHtml(shortHash(tx.recipient))}</a>
        </span>
        <span class="amount-badge">${microAruToAru(tx.amount)} ARU</span>
      </a>
    `;
  }

  // ── Detail Row ───────────────────────────────────────────────────────────────
  function detailRow(label, valueHtml) {
    return `
      <div class="detail-row">
        <span class="detail-label">${escHtml(label)}</span>
        <span class="detail-value">${valueHtml}</span>
      </div>
    `;
  }

  function detailRowMono(label, value) {
    return `
      <div class="detail-row">
        <span class="detail-label">${escHtml(label)}</span>
        <span class="detail-value mono">${escHtml(value)}</span>
      </div>
    `;
  }

  // ── SVG Chart Plotter ────────────────────────────────────────────────────────
  function drawDifficultyChart(blocks) {
    const svg = document.getElementById('difficulty-chart');
    if (!svg) return;

    const points = blocks.map(b => b.difficulty).reverse();
    if (points.length < 2) return;

    const min = Math.min(...points);
    const max = Math.max(...points);
    const range = max - min || 1;

    const width = 500;
    const height = 150;
    const startX = 50;
    const startY = 20;

    const coords = points.map((p, i) => {
      const x = startX + (i / (points.length - 1)) * width;
      const y = startY + height - ((p - min) / range) * height;
      return `${x},${y}`;
    });

    const path = document.getElementById('chart-path');
    if (path) {
      path.setAttribute('d', `M ${coords.join(' L ')}`);
    }
  }

  // ── Dashboard ────────────────────────────────────────────────────────────────
  async function loadDashboard() {
    // 1. Load Stats
    try {
      const stats = await api.stats();
      setText('stat-height', numFmt(stats.height));
      setText('stat-txs', numFmt(stats.total_tx_count));
      setText('stat-time', timeAgo(stats.last_block_time));
      setText('stat-peers', stats.node ? numFmt(stats.node.peer_count) : '1');
    } catch (err) {
      console.warn('Stats error:', err);
    }

    // 2. Load Latest Block Detail Card
    try {
      const latest = await api.blockLatest();
      if (latest) {
        const html = `
          ${detailRow('Height', `#${numFmt(latest.height)}`)}
          ${detailRowMono('Hash', latest.hash)}
          ${detailRow('Timestamp', timestamp(latest.timestamp))}
          ${detailRow('Difficulty', numFmt(latest.difficulty))}
          ${detailRow('Nonce', numFmt(latest.nonce))}
        `;
        setHtml('latest-block-card', html);
      }
    } catch (err) {
      showError('latest-block-card', err.message);
    }

    // 3. Load Latest Transactions List
    try {
      const latest = await api.blockLatest();
      const txs = latest.transactions || [];
      if (txs.length === 0) {
        setHtml('latest-txs-list', '<div class="empty-state">No transactions in the latest block.</div>');
      } else {
        setHtml('latest-txs-list', txs.slice(0, 5).map(txListItem).join(''));
      }
    } catch (err) {
      showError('latest-txs-list', err.message);
    }

    // 4. Draw difficulty chart using last 10 blocks
    try {
      const data = await api.blocks(10, 0);
      const blocks = data.items || data;
      drawDifficultyChart(blocks);
    } catch (err) {
      console.warn('Failed to load blocks for chart:', err);
    }
  }

  function initDashboard() {
    setupSearch('hero-search-form', 'hero-search-input');
    loadDashboard();
    setInterval(loadDashboard, REFRESH_INTERVAL_MS);
  }

  // ── Block Detail ─────────────────────────────────────────────────────────────
  async function loadBlockDetail() {
    const hash   = getParam('hash');
    const height = getParam('height');

    let block;
    try {
      if (hash) {
        block = await api.blockByHash(hash);
      } else if (height !== null && height !== undefined) {
        block = await api.blockByHeight(Number(height));
      } else {
        block = await api.blockLatest();
      }
    } catch (err) {
      showError('block-detail-card', err.message);
      showError('block-txs-list', '');
      return;
    }

    setText('block-height-heading', numFmt(block.height));
    setText('block-hash-heading', block.hash);
    setText('breadcrumb-id', `#${block.height}`);

    const html = `
      ${detailRow('Height',       `<a href="block.html?height=${block.height - 1}">#${numFmt(block.height)}</a>`)}
      ${detailRowMono('Hash',     block.hash)}
      ${detailRowMono('Previous', `<a href="block.html?hash=${block.prev_hash}">${shortHash(block.prev_hash)}</a>`)}
      ${detailRowMono('Merkle Root', block.merkle_root)}
      ${detailRowMono('State Root',  block.state_root)}
      ${detailRow('Timestamp',    escHtml(timestamp(block.timestamp)))}
      ${detailRow('Difficulty',   numFmt(block.difficulty))}
      ${detailRow('Nonce',        numFmt(block.nonce))}
      ${detailRow('Version',      String(block.version))}
      ${detailRow('Transactions', String(block.tx_count))}
    `;
    setHtml('block-detail-card', html);
    setText('block-tx-count', String(block.tx_count));

    const txs = block.transactions || [];
    if (txs.length === 0) {
      setHtml('block-txs-list', '<div class="empty-state">No transactions in this block.</div>');
    } else {
      setHtml('block-txs-list', txs.map(txListItem).join(''));
    }
  }

  function initBlockDetail() {
    setupSearch('hero-search-form', 'hero-search-input');
    loadBlockDetail();
  }

  // ── Transaction Detail ───────────────────────────────────────────────────────
  async function loadTxDetail() {
    const hash = getParam('hash');
    if (!hash) {
      showError('tx-detail-card', 'No transaction hash provided.');
      return;
    }

    let tx;
    try {
      tx = await api.transaction(hash);
    } catch (err) {
      showError('tx-detail-card', err.message);
      return;
    }

    setText('tx-hash-heading', tx.hash);
    setText('breadcrumb-id', shortHash(tx.hash));

    const sigLabel = tx.sig_type === 0 ? 'Ed25519' : 'secp256k1';
    const html = `
      ${detailRow('Status', '<span class="tag-confirmed">✓ Confirmed</span>')}
      ${detailRowMono('Hash', tx.hash)}
      ${detailRow('Block', `<a href="block.html?height=${tx.block_height}">#${numFmt(tx.block_height)}</a>`)}
      ${detailRowMono('Block Hash', `<a href="block.html?hash=${tx.block_hash}">${shortHash(tx.block_hash)}</a>`)}
      ${detailRow('Index in Block', String(tx.tx_index))}
      ${detailRow('From', `<a href="address.html?addr=${encodeURIComponent(tx.sender)}" class="mono">${escHtml(shortHash(tx.sender))}</a>`)}
      ${detailRow('To',   `<a href="address.html?addr=${encodeURIComponent(tx.recipient)}" class="mono">${escHtml(shortHash(tx.recipient))}</a>`)}
      ${detailRow('Amount',    `${microAruToAru(tx.amount)} ARU`)}
      ${detailRow('Fee',       `${microAruToAru(tx.fee)} ARU`)}
      ${detailRow('Nonce',     numFmt(tx.nonce_val))}
      ${detailRow('Gas Limit', numFmt(tx.gas_limit))}
      ${detailRow('Gas Price', numFmt(tx.gas_price))}
      ${detailRow('Signature', sigLabel)}
      ${detailRow('Has Data',  tx.has_data ? 'Yes' : 'No')}
    `;
    setHtml('tx-detail-card', html);
  }

  function initTxDetail() {
    setupSearch('hero-search-form', 'hero-search-input');
    loadTxDetail();
  }

  // ── Address Detail ───────────────────────────────────────────────────────────
  async function loadAddressDetail() {
    const addr = getParam('addr');
    if (!addr) {
      showError('addr-txs-list', 'No address provided.');
      return;
    }

    setText('addr-heading', addr);
    setText('breadcrumb-id', shortHash(addr));

    let data;
    try {
      data = await api.address(addr, 20, 0);
    } catch (err) {
      showError('addr-txs-list', err.message);
      return;
    }

    setText('addr-balance', `${microAruToAru(data.balance)} ARU`);
    setText('addr-nonce', numFmt(data.nonce));
    setText('addr-updated', numFmt(data.updated_at_block));

    const txs = data.transactions || [];
    if (txs.length === 0) {
      setHtml('addr-txs-list', '<div class="empty-state">No transactions found for this address.</div>');
    } else {
      setHtml('addr-txs-list', txs.map(txListItem).join(''));
    }
  }

  function initAddressDetail() {
    setupSearch('hero-search-form', 'hero-search-input');
    loadAddressDetail();
  }

  // ── Network Metrics Page ───────────────────────────────────────────────────────
  async function initNetworkPage() {
    try {
      const stats = await api.stats();
      const html = `
        ${detailRow('Network Name', stats.node ? stats.node.network : 'Sumatera')}
        ${detailRow('Chain ID', stats.node ? String(stats.node.chain_id) : '7777')}
        ${detailRow('Synced', stats.node && stats.node.synced ? '<span class="health-active">✓ Synced</span>' : 'Standalone')}
        ${detailRow('Uptime', stats.node ? `${numFmt(Math.floor(stats.node.uptime_seconds / 3600))} hours` : '—')}
        ${detailRow('Active Peers', stats.node ? numFmt(stats.node.peer_count) : '1')}
        ${detailRow('Version', stats.node ? stats.node.version : '0.1.0')}
        ${detailRow('Best Block Height', `#${numFmt(stats.height)}`)}
        ${detailRowMono('Best Block Hash', stats.best_hash)}
      `;
      setHtml('network-metrics-detail', html);
    } catch (err) {
      showError('network-metrics-detail', err.message);
    }
  }

  // ── Peers Page ───────────────────────────────────────────────────────────────
  async function initPeersPage() {
    try {
      const data = await api.peers();
      const peers = data.peers || [];
      if (peers.length === 0) {
        setHtml('peers-list-panel', `
          <div class="empty-state">No active peers connected. Nodes operate in standalone genesis mode.</div>
        `);
      } else {
        const rows = peers.map((p, i) => `
          <tr>
            <td class="mono">#${i + 1}</td>
            <td class="mono">${escHtml(p)}</td>
            <td>Full Node</td>
            <td><span class="health-active">Active</span></td>
          </tr>
        `).join('');

        setHtml('peers-list-panel', `
          <table class="grid-table">
            <thead>
              <tr>
                <th>Index</th>
                <th>Peer Address (P2P)</th>
                <th>Capabilities</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              ${rows}
            </tbody>
          </table>
        `);
      }
    } catch (err) {
      showError('peers-list-panel', err.message);
    }
  }

  // ── Nodes/Validators Page ─────────────────────────────────────────────────────
  async function initNodesPage() {
    try {
      const data = await api.validators();
      const count = data.active_validators_count || 1;
      
      const rows = `
        <tr>
          <td class="mono">#1 (Local Node)</td>
          <td class="mono">${escHtml(data.reward_address || 'sum1faucetaddressxxxxxxxxxxxxxxxxxxxxxxxxxx')}</td>
          <td>10,000 ARU (Min Stake)</td>
          <td><span class="health-active">Active Validator</span></td>
        </tr>
      `;

      setHtml('nodes-list-panel', `
        <table class="grid-table">
          <thead>
            <tr>
              <th>Validator</th>
              <th>Reward Address</th>
              <th>Stake Weight</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            ${rows}
          </tbody>
        </table>
      `);
    } catch (err) {
      showError('nodes-list-panel', err.message);
    }
  }

  // ── Public API ───────────────────────────────────────────────────────────────
  return {
    initDashboard,
    initBlockDetail,
    initTxDetail,
    initAddressDetail,
    initNetworkPage,
    initPeersPage,
    initNodesPage,
  };
})();
