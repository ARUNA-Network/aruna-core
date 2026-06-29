import { getStatus, getNetwork } from '../../services/api';
import { renderHeader } from '../../components/Header';
import { renderFooter } from '../../components/Footer';
import { numFmt, escHtml } from '../../utils/format';

const el = (id: string) => document.getElementById(id);

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

async function loadNetworkStatus() {
  try {
    const stats = await getStatus();
    const html = `
      ${detailRow('Network Name', stats.node ? stats.node.network : 'Sumatera')}
      ${detailRow('Chain ID', stats.node ? String(stats.node.chain_id) : '7777')}
      ${detailRow('Synced', stats.node && stats.node.synced ? '<span class="health-active">✓ Synced</span>' : 'Standalone')}
      ${detailRow('Uptime', stats.node ? `${numFmt(Math.floor(stats.node.uptime_seconds / 3600))} hours` : '—')}
      ${detailRow('Active Peers', stats.node ? numFmt(stats.node.peer_count) : '0')}
      ${detailRow('Version', stats.node ? stats.node.version : '0.1.0')}
      ${detailRow('Best Block Height', `#${numFmt(stats.height)}`)}
      ${detailRowMono('Best Block Hash', stats.best_hash)}
    `;
    setHtml('network-metrics-detail', html);
  } catch (err) {
    showError('network-metrics-detail', (err as Error).message);
  }
}

async function loadPeersList() {
  try {
    const data = await getNetwork();
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
    showError('peers-list-panel', (err as Error).message);
  }
}

export function initNetwork(): void {
  renderHeader('network');
  renderFooter();
  loadNetworkStatus();
}

export function initPeers(): void {
  renderHeader('peers');
  renderFooter();
  loadPeersList();
}
