export function renderHeader(activePage: string): void {
  const headerEl = document.querySelector('nav.navbar');
  if (!headerEl) return;

  const pages = [
    { name: 'Home', url: 'index.html', key: 'home' },
    { name: 'Block', url: 'block.html', key: 'block' },
    { name: 'Transaction', url: 'tx.html', key: 'tx' },
    { name: 'Address', url: 'address.html', key: 'address' },
    { name: 'Network', url: 'network.html', key: 'network' },
    { name: 'Supply', url: 'supply.html', key: 'supply' },
    { name: 'Peers', url: 'peers.html', key: 'peers' },
    { name: 'Nodes', url: 'nodes.html', key: 'nodes' },
  ];

  const menuHtml = pages
    .map(
      (p) => `
      <li role="none">
        <a href="${p.url}" class="nav-link ${activePage === p.key ? 'active' : ''}" role="menuitem">${p.name}</a>
      </li>
    `
    )
    .join('');

  headerEl.innerHTML = `
    <div class="nav-inner">
      <a href="index.html" class="nav-logo" aria-label="ARUNA Explorer Home">
        <div class="logo-icon">⬡</div>
        <div class="logo-text">
          <span class="logo-name">ARUNA</span>
          <span class="logo-sub">Explorer</span>
        </div>
      </a>
      <div class="nav-badge">Sumatera Testnet</div>
      <ul class="nav-menu" role="menubar">
        ${menuHtml}
      </ul>
    </div>
  `;
}
