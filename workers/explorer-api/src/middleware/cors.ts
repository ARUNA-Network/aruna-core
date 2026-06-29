/**
 * CORS handling middleware for the Explorer API.
 */

export const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type',
};

export function handleOptions(request: Request): Response | null {
  if (request.method === 'OPTIONS') {
    return new Response(null, {
      status: 204,
      headers: corsHeaders,
    });
  }
  return null;
}

export function applyCors(response: Response): Response {
  const newHeaders = new Headers(response.headers);
  for (const [key, val] of Object.entries(corsHeaders)) {
    newHeaders.set(key, val);
  }
  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers: newHeaders,
  });
}
