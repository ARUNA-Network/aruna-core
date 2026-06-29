export function renderFooter(): void {
  const footerEl = document.querySelector('footer.footer');
  if (!footerEl) return;

  footerEl.innerHTML = `
    <div class="container footer-inner">
      <span>© 2026 ARUNA Network — <em>Dari Rakyat. Oleh Rakyat. Untuk Rakyat.</em></span>
      <span class="footer-links">
        <a href="https://github.com/ARUNA-Network/aruna-core" target="_blank" rel="noopener">GitHub</a>
      </span>
    </div>
  `;
}
