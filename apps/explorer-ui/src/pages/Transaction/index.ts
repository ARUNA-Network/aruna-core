import { getTransaction } from '../../services/api';
import { renderHeader } from '../../components/Header';
import { renderFooter } from '../../components/Footer';
import { setupSearchBar } from '../../components/SearchBar';
import { numFmt, escHtml, shortHash, microAruToAru } from '../../utils/format';

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

async function loadTxDetail() {
  const hash = getParam('hash');
  if (!hash) {
    showError('tx-detail-card', 'No transaction hash provided.');
    return;
  }

  try {
    const tx = await getTransaction(hash);
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
      ${detailRow('To', `<a href="address.html?addr=${encodeURIComponent(tx.recipient)}" class="mono">${escHtml(shortHash(tx.recipient))}</a>`)}
      ${detailRow('Amount', `${microAruToAru(tx.amount)} ARU`)}
      ${detailRow('Fee', `${microAruToAru(tx.fee)} ARU`)}
      ${detailRow('Nonce', numFmt(tx.nonce_val))}
      ${detailRow('Gas Limit', numFmt(tx.gas_limit))}
      ${detailRow('Gas Price', numFmt(tx.gas_price))}
      ${detailRow('Signature', sigLabel)}
      ${detailRow('Has Data', tx.has_data ? 'Yes' : 'No')}
    `;
    setHtml('tx-detail-card', html);
  } catch (err) {
    showError('tx-detail-card', (err as Error).message);
  }
}

export function initTx(): void {
  renderHeader('tx');
  renderFooter();
  setupSearchBar('hero-search-form', 'hero-search-input');
  loadTxDetail();
}
