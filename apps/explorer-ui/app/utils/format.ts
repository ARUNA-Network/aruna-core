export function shortHash(h = ''): string {
  if (!h || h.length < 16) return h;
  return h.slice(0, 8) + '…' + h.slice(-6);
}

export function microAruToAru(micro: number | string): string {
  return (Number(micro) / 1000000).toLocaleString('en-US', { maximumFractionDigits: 6 });
}

export function timeAgo(unixSecs: number | string): string {
  const diff = Math.floor(Date.now() / 1000) - Number(unixSecs);
  if (diff < 5)   return 'just now';
  if (diff < 60)  return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return new Date(Number(unixSecs) * 1000).toLocaleDateString();
}

export function timestamp(unixSecs: number | string): string {
  return new Date(Number(unixSecs) * 1000).toLocaleString();
}

export function numFmt(n: number | string): string {
  return Number(n).toLocaleString('en-US');
}

export function escHtml(s: string | number): string {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
