/**
 * Basic request logging middleware for the Edge Worker.
 */

export function logRequest(method: string, pathname: string, startMs: number, status: number): void {
  const duration = Date.now() - startMs;
  console.log(`[Explorer API] ${method} ${pathname} - Status: ${status} - Timing: ${duration}ms`);
}
