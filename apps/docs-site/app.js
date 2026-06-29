/**
 * ARUNA Docs Site — Client Module
 *
 * Manages documentation navigation tabs, code block stubs, and content filtering.
 */

document.addEventListener('DOMContentLoaded', () => {
  const navLinks = document.querySelectorAll('.nav-link');
  const tabPanes = document.querySelectorAll('.tab-pane');
  const searchInput = document.getElementById('docs-search');

  // 1. Navigation Tab Controller
  navLinks.forEach(link => {
    link.addEventListener('click', () => {
      const targetTab = link.getAttribute('data-tab');

      navLinks.forEach(l => l.classList.remove('active'));
      tabPanes.forEach(pane => pane.classList.remove('active-pane'));

      link.classList.add('active');
      const pane = document.getElementById(targetTab);
      if (pane) {
        pane.classList.add('active-pane');
      }
    });
  });

  // 2. Search Text Filtering
  searchInput.addEventListener('input', (e) => {
    const query = e.target.value.toLowerCase().trim();

    navLinks.forEach(link => {
      const text = link.textContent.toLowerCase();
      const tabId = link.getAttribute('data-tab');
      const pane = document.getElementById(tabId);
      const content = pane ? pane.textContent.toLowerCase() : '';

      if (text.includes(query) || content.includes(query)) {
        link.classList.remove('hidden');
      } else {
        link.classList.add('hidden');
      }
    });
  });
});
