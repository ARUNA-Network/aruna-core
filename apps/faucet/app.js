/**
 * ARUNA Faucet — Client Application
 *
 * Communicates exclusively with Faucet API Edge Worker.
 */

document.addEventListener('DOMContentLoaded', () => {
  const form = document.getElementById('faucet-form');
  const addressInput = document.getElementById('address-input');
  const submitBtn = document.getElementById('submit-btn');
  const statusPanel = document.getElementById('status-panel');
  const statusIcon = document.getElementById('status-icon');
  const statusMsg = document.getElementById('status-message');
  const statusTx = document.getElementById('status-tx-hash');

  const API_ENDPOINT = window.ARUNA_FAUCET_API_URL || 'http://127.0.0.1:8787'; // Faucet worker URL

  form.addEventListener('submit', async (e) => {
    e.preventDefault();

    const address = addressInput.value.trim();
    if (!address.startsWith('sum1')) {
      showStatus('error', '❌ Invalid address prefix (must start with sum1)');
      return;
    }

    // Capture turnstile response token
    const turnstileResponse = window.turnstile?.getResponse() || 'mock-turnstile-token';

    showStatus('loading', '⏳ Requesting testnet tokens...');
    submitBtn.disabled = true;

    try {
      const response = await fetch(API_ENDPOINT, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          address: address,
          captchaToken: turnstileResponse
        })
      });

      const result = await response.json();

      if (!response.ok) {
        throw new Error(result.error || `HTTP ${response.status}`);
      }

      showStatus('success', `✅ Successfully sent ${result.amount} ARU to address!`, result.tx_hash);
      form.reset();

    } catch (err) {
      showStatus('error', `❌ Request failed: ${err.message}`);
    } finally {
      submitBtn.disabled = false;
      // Reset Turnstile captcha if exists
      if (window.turnstile) {
        window.turnstile.reset();
      }
    }
  });

  function showStatus(type, message, txHash = '') {
    statusPanel.classList.remove('hidden', 'success', 'error');
    statusTx.classList.add('hidden');

    if (type === 'loading') {
      statusIcon.textContent = '⏳';
      statusMsg.textContent = message;
    } else if (type === 'success') {
      statusPanel.classList.add('success');
      statusIcon.textContent = '🎉';
      statusMsg.textContent = message;
      if (txHash) {
        statusTx.textContent = `Tx Hash: ${txHash}`;
        statusTx.classList.remove('hidden');
      }
    } else if (type === 'error') {
      statusPanel.classList.add('error');
      statusIcon.textContent = '⚠️';
      statusMsg.textContent = message;
    }
  }
});
