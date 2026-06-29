import { api } from '../../services/api';
import { renderHeader } from '../../components/Header';
import { renderFooter } from '../../components/Footer';
import { setupSearchBar } from '../../components/SearchBar';
import { numFmt, timeAgo, timestamp, escHtml, shortHash, microAruToAru } from '../../utils/format';
import { Block, Transaction } from '../../types';

// Global elements
const el = (id: string) => document.getElementById(id);

function setText(id: string, text: string) {
  const e = el(id);
  if (e) { e.textContent = text; e.classList.remove('skeleton'); }
}

function setHtml(id: string, html: string) {
  const e = el(id);
  if (e) e.innerHTML = html;
}

function showError(containerId: string, message: string) {
  setHtml(containerId, `
    <div class="error-state">
      <span class="error-icon">⚠️</span>
      <p>Failed to load data</p>
      <span class="error-msg">${escHtml(message)}</span>
    </div>
  `);
}



function txListItem(tx: Transaction): string {
  return `
    <a href="tx.html?hash=${encodeURIComponent(tx.hash)}" class="list-item tx-row" role="listitem" aria-label="Transaction ${tx.hash}">
      <span class="hash-short">${escHtml(shortHash(tx.hash))}</span>
      <span class="item-meta">
        <a href="address.html?addr=${encodeURIComponent(tx.sender)}" onclick="event.stopPropagation()">${escHtml(shortHash(tx.sender))}</a>
        → <a href="address.html?addr=${encodeURIComponent(tx.recipient)}" onclick="event.stopPropagation()">${escHtml(shortHash(tx.recipient))}</a>
      </span>
      <span class="amount-badge">${microAruToAru(tx.amount)} ARU</span>
    </a>
  `;
}

function detailRow(label: string, valueHtml: string): string {
  return `
    <div class="detail-row">
      <span class="detail-label">${escHtml(label)}</span>
      <span class="detail-value">${valueHtml}</span>
    </div>
  `;
}

function detailRowMono(label: string, value: string): string {
  return `
    <div class="detail-row">
      <span class="detail-label">${escHtml(label)}</span>
      <span class="detail-value mono">${escHtml(value)}</span>
    </div>
  `;
}

function drawDifficultyChart(blocks: Block[]): void {
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

async function loadDashboard() {
  // 1. Load Stats
  try {
    const stats = await api.stats();
    setText('stat-height', numFmt(stats.height));
    setText('stat-txs', numFmt(stats.total_tx_count));
    setText('stat-time', timeAgo(stats.last_block_time));
    setText('stat-peers', stats.node ? numFmt(stats.node.peer_count) : '0');
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
    showError('latest-block-card', (err as Error).message);
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
    showError('latest-txs-list', (err as Error).message);
  }

  // 4. Draw difficulty chart
  try {
    const data = await api.blocks(10, 0);
    drawDifficultyChart(data);
  } catch (err) {
    console.warn('Failed to load blocks for chart:', err);
  }
}

export function initHome(): void {
  renderHeader('home');
  renderFooter();
  setupSearchBar('hero-search-form', 'hero-search-input');
  loadDashboard();
  setInterval(loadDashboard, 12000);
}

if (typeof document !== 'undefined' && document.getElementById('stat-height')) {
  initHome();
}
