import { getAddress } from '../../services/api';
import { renderHeader } from '../../components/Header';
import { renderFooter } from '../../components/Footer';
import { setupSearchBar } from '../../components/SearchBar';
import { numFmt, escHtml, shortHash, microAruToAru } from '../../utils/format';
import { Transaction } from '../../types';

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
    <a href="/transaction/${encodeURIComponent(tx.hash)}" class="list-item tx-row" role="listitem" aria-label="Transaction ${tx.hash}">
      <span class="hash-short">${escHtml(shortHash(tx.hash))}</span>
      <span class="item-meta">
        <a href="/address/${encodeURIComponent(tx.sender)}" onclick="event.stopPropagation()">${escHtml(shortHash(tx.sender))}</a>
        → <a href="/address/${encodeURIComponent(tx.recipient)}" onclick="event.stopPropagation()">${escHtml(shortHash(tx.recipient))}</a>
      </span>
      <span class="amount-badge">${microAruToAru(tx.amount)} ARU</span>
    </a>
  `;
}

async function loadAddressDetail() {
  const path = window.location.pathname;
  if (!path.startsWith('/address/')) {
    showError('addr-txs-list', 'Invalid address path.');
    return;
  }
  const addr = path.substring('/address/'.length).trim();
  if (!addr) {
    showError('addr-txs-list', 'No address provided.');
    return;
  }

  setText('addr-heading', addr);
  setText('breadcrumb-id', shortHash(addr));

  try {
    const data = await getAddress(addr, 20, 0);
    setText('addr-balance', `${microAruToAru(data.balance)} ARU`);
    setText('addr-nonce', numFmt(data.nonce));
    setText('addr-updated', numFmt(data.updated_at_block));

    const txs = data.transactions || [];
    if (txs.length === 0) {
      setHtml('addr-txs-list', '<div class="empty-state">No transactions found for this address.</div>');
    } else {
      setHtml('addr-txs-list', txs.map(txListItem).join(''));
    }
  } catch (err) {
    showError('addr-txs-list', (err as Error).message);
  }
}

export function initAddress(): void {
  renderHeader('address');
  renderFooter();
  setupSearchBar('hero-search-form', 'hero-search-input');
  loadAddressDetail();
}
