import { getNetwork } from '../../services/api';
import { renderHeader } from '../../components/Header';
import { renderFooter } from '../../components/Footer';
import { escHtml } from '../../utils/format';

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

async function loadNodesList() {
  try {
    const data = await getNetwork();
    const validators = data.validators || { active_validators_count: 1, reward_address: "" };

    const rows = `
      <tr>
        <td class="mono">#1 (Local Node)</td>
        <td class="mono">
          <a href="/address/${encodeURIComponent(validators.reward_address || 'sum1faucetaddressxxxxxxxxxxxxxxxxxxxxxxxxxx')}">
            ${escHtml(validators.reward_address || 'sum1faucetaddressxxxxxxxxxxxxxxxxxxxxxxxxxx')}
          </a>
        </td>
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
    showError('nodes-list-panel', (err as Error).message);
  }
}

export function initValidators(): void {
  renderHeader('nodes');
  renderFooter();
  loadNodesList();
}
