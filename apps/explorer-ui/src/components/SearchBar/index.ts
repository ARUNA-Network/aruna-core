import { search } from '../../services/api';

export function setupSearchBar(formId: string, inputId: string): void {
  const form = document.getElementById(formId) as HTMLFormElement | null;
  if (!form) return;

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const input = document.getElementById(inputId) as HTMLInputElement | null;
    const q = input?.value?.trim();
    if (!q) return;

    try {
      const results = await search(q);
      if (results.length === 0) {
        alert('No results found for: ' + q);
        return;
      }
      const first = results[0];
      if (first.kind === 'block') {
        window.location.href = `block.html?hash=${encodeURIComponent(first.value)}`;
      } else if (first.kind === 'transaction') {
        window.location.href = `tx.html?hash=${encodeURIComponent(first.value)}`;
      } else if (first.kind === 'address') {
        window.location.href = `address.html?addr=${encodeURIComponent(first.value)}`;
      }
    } catch (err) {
      // Fallbacks if search endpoint fails or isn't populated
      if (/^\d+$/.test(q)) {
        window.location.href = `block.html?height=${encodeURIComponent(q)}`;
      } else if (q.length === 64) {
        window.location.href = `block.html?hash=${encodeURIComponent(q)}`;
      } else {
        window.location.href = `address.html?addr=${encodeURIComponent(q)}`;
      }
    }
  });
}
