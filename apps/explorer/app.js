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
  const API_BASE = (window.ARUNA_API_URL || 'http://127.0.0.1:3000') + '/api/v1';
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
    blocks:         (l, o)     => apiFetch(`/blocks?limit=${l}&offset=${o}`),
    blockLatest:    ()          => apiFetch('/block/latest'),
    blockByHeight:  (n)         => apiFetch(`/block/height/${n}`),
    blockByHash:    (h)         => apiFetch(`/block/hash/${h}`),
    transaction:    (hash)      => apiFetch(`/transaction/${hash}`),
    address:        (addr, l, o) => apiFetch(`/address/${addr}?limit=${l}&offset=${o}`),
    search:         (q)         => apiFetch(`/search?q=${encodeURIComponent(q)}`),
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
          // Block hash → block detail
          navigate(`block.html?hash=${encodeURIComponent(first.value)}`);
        } else if (first.kind === 'transaction') {
          navigate(`tx.html?hash=${encodeURIComponent(first.value)}`);
        } else if (first.kind === 'address') {
          navigate(`address.html?addr=${encodeURIComponent(first.value)}`);
        }
      } catch (err) {
        // If search fails, try to guess the type by format
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

  // ── Dashboard ────────────────────────────────────────────────────────────────
  async function loadDashboard() {
    // Load stats
    try {
      const stats = await api.stats();
      setText('stat-height', numFmt(stats.height));
      setText('stat-txs', numFmt(stats.total_tx_count));
      setText('stat-time', timeAgo(stats.last_block_time));
      const hashEl = el('stat-best');
      if (hashEl) {
        hashEl.querySelector('.stat-value').textContent = shortHash(stats.best_hash);
        hashEl.querySelector('.stat-value').classList.remove('skeleton');
      }
    } catch (err) {
      console.warn('Stats error:', err);
    }

    // Load recent blocks
    try {
      const data = await api.blocks(8, 0);
      const blocks = data.items || data;
      if (blocks.length === 0) {
        setHtml('blocks-list', '<div class="empty-state">No blocks indexed yet.</div>');
      } else {
        setHtml('blocks-list', blocks.map(blockListItem).join(''));
      }
      // View-all link: deeplink to highest block
      if (blocks.length > 0) {
        const latest = blocks[0];
        el('view-all-blocks').href = `block.html?height=${latest.height}`;
      }
    } catch (err) {
      showError('blocks-list', err.message);
    }

    // Load recent transactions (from latest block)
    try {
      const latest = await api.blockLatest();
      const txs = latest.transactions || [];
      if (txs.length === 0) {
        setHtml('txs-list', '<div class="empty-state">No transactions yet.</div>');
      } else {
        setHtml('txs-list', txs.slice(0, 8).map(txListItem).join(''));
        if (txs.length > 0) {
          el('view-all-txs').href = `tx.html?hash=${encodeURIComponent(txs[0].hash)}`;
        }
      }
    } catch (err) {
      showError('txs-list', err.message);
    }
  }

  function initDashboard() {
    setupSearch('nav-search-form', 'nav-search-input');
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
      } else if (height !== null) {
        block = await api.blockByHeight(Number(height));
      } else {
        block = await api.blockLatest();
      }
    } catch (err) {
      showError('block-detail-card', err.message);
      showError('block-txs-list', '');
      return;
    }

    // Update heading
    setText('block-height-heading', numFmt(block.height));
    setText('block-hash-heading', block.hash);
    setText('breadcrumb-id', `#${block.height}`);

    // Build detail card
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

    // Transaction count badge
    setText('block-tx-count', String(block.tx_count));

    // Transaction list
    const txs = block.transactions || [];
    if (txs.length === 0) {
      setHtml('block-txs-list', '<div class="empty-state">No transactions in this block.</div>');
    } else {
      setHtml('block-txs-list', txs.map(txListItem).join(''));
    }
  }

  function initBlockDetail() {
    setupSearch('nav-search-form', 'nav-search-input');
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
    setupSearch('nav-search-form', 'nav-search-input');
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
    setupSearch('nav-search-form', 'nav-search-input');
    loadAddressDetail();
  }

  // ── Public API ───────────────────────────────────────────────────────────────
  return {
    initDashboard,
    initBlockDetail,
    initTxDetail,
    initAddressDetail,
  };
})();
