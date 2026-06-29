import { getStatus } from '../../services/api';
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

async function loadSupplyStats() {
  try {
    const stats = await getStatus();
    // Calculations according to ARUNA Economic Constitution
    const maxSupply = 1_000_000_000; // 1 Billion ARU
    
    // Premine: 1.5% (15,000,000 ARU)
    const premine = 15_000_000;
    
    // Simple block reward sum: 25 ARU per block
    const blockReward = 25;
    const circulating = premine + (stats.height * blockReward);
    
    const html = `
      ${detailRow('Max Supply Cap', `${numFmt(maxSupply)} ARU`)}
      ${detailRow('Genesis Premine (1.5%)', `${numFmt(premine)} ARU`)}
      ${detailRow('Block Reward', `${blockReward} ARU`)}
      ${detailRow('Current Circulating Supply', `<span class="text-glow">${numFmt(circulating)} ARU</span>`)}
      ${detailRow('Halving Interval', '4,204,800 blocks (~4 Years)')}
    `;
    setHtml('supply-metrics-detail', html);
  } catch (err) {
    showError('supply-metrics-detail', (err as Error).message);
  }
}

export function initSupply(): void {
  renderHeader('supply');
  renderFooter();
  loadSupplyStats();
}
