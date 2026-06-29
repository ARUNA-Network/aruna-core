# ARUNA Network Explorer UI

Static block explorer frontend for the ARUNA Network.

## Architecture

See [ADR-0017](../../docs/adr/adr-0017-explorer-architecture.md).

All data is fetched from the `aruna-explorer` REST API (`/api/v1/*`). No direct blockchain or database access.

## Pages

| File            | Purpose                             |
|-----------------|-------------------------------------|
| `index.html`    | Dashboard: chain stats, recent blocks & transactions |
| `block.html`    | Block detail: header fields, transaction list |
| `tx.html`       | Transaction detail: sender, recipient, amount, block |
| `address.html`  | Address detail: balance, nonce, tx history |

## Running Locally

Open `index.html` directly in a browser (file://) **or** serve via any static file server:

```bash
# Quick local test with Python
python3 -m http.server 8000

# Or Node.js
npx serve .
```

## API Configuration

By default the UI connects to `http://127.0.0.1:3000/api/v1`.

To connect to a remote API, set `window.ARUNA_API_URL` before loading `app.js`:

```html
<script>window.ARUNA_API_URL = 'https://explorer-api.aruna.network';</script>
<script src="app.js"></script>
```

## Technology Stack

- HTML5 (semantic, accessible)
- Vanilla CSS (custom design system, no framework)
- Vanilla JavaScript (ES2020, no framework, no bundler)
- Google Fonts: Outfit + JetBrains Mono
