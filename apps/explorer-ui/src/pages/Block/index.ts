import { api } from '../../services/api';
import { renderHeader } from '../../components/Header';
import { renderFooter } from '../../components/Footer';
import { setupSearchBar } from '../../components/SearchBar';
import { numFmt, timestamp, escHtml, shortHash, microAruToAru } from '../../utils/format';
import { Block, Transaction } from '../../types';

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

function getParam(name: string): string | null {
  return new URLSearchParams(window.location.search).get(name);
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

async function loadBlockDetail() {
  const hash = getParam('hash');
  const height = getParam('height');

  let block: Block;
  try {
    if (hash) {
      block = await api.blockByHash(hash);
    } else if (height !== null && height !== undefined) {
      block = await api.blockByHeight(Number(height));
    } else {
      block = await api.blockLatest();
    }
  } catch (err) {
    showError('block-detail-card', (err as Error).message);
    showError('block-txs-list', '');
    return;
  }

  setText('block-height-heading', numFmt(block.height));
  setText('block-hash-heading', block.hash);
  setText('breadcrumb-id', `#${block.height}`);

  const html = `
    ${detailRow('Height', `<a href="block.html?height=${block.height - 1}">#${numFmt(block.height)}</a>`)}
    ${detailRowMono('Hash', block.hash)}
    ${detailRowMono('Previous', `<a href="block.html?hash=${block.prev_hash}">${shortHash(block.prev_hash)}</a>`)}
    ${detailRowMono('Merkle Root', block.merkle_root)}
    ${detailRowMono('State Root', block.state_root)}
    ${detailRow('Timestamp', escHtml(timestamp(block.timestamp)))}
    ${detailRow('Difficulty', numFmt(block.difficulty))}
    ${detailRow('Nonce', numFmt(block.nonce))}
    ${detailRow('Version', String(block.version))}
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

export function initBlock(): void {
  renderHeader('block');
  renderFooter();
  setupSearchBar('hero-search-form', 'hero-search-input');
  loadBlockDetail();
}
