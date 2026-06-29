export default {
  async fetch(request: Request, env: any, ctx: any): Promise<Response> {
    const url = new URL(request.url);
    const pathname = url.pathname;

    // Clean routing maps to the compiled static HTML pages
    if (pathname === '/' || pathname === '/blocks' || pathname === '/transactions' || pathname === '/search') {
      url.pathname = '/index.html';
    } else if (pathname.startsWith('/block/hash/')) {
      url.pathname = '/block.html';
    } else if (pathname.startsWith('/block/')) {
      url.pathname = '/block.html';
    } else if (pathname.startsWith('/transaction/')) {
      url.pathname = '/tx.html';
    } else if (pathname.startsWith('/address/')) {
      url.pathname = '/address.html';
    } else if (pathname === '/network') {
      url.pathname = '/network.html';
    } else if (pathname === '/stats' || pathname === '/supply') {
      url.pathname = '/supply.html';
    } else if (pathname === '/peers') {
      url.pathname = '/peers.html';
    } else if (pathname === '/nodes' || pathname === '/validators') {
      url.pathname = '/nodes.html';
    } else {
      // Catch-all fall back to index.html
      url.pathname = '/index.html';
    }

    return env.ASSETS.fetch(new Request(url.toString(), request));
  },
};
