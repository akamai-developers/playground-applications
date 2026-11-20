import * as Variables from '@spinframework/spin-variables';

// Proxies every GET request to the origin and adds a custom response header
async function handle(request: Request): Promise<Response> {
  if (request.method !== 'GET') {
    return new Response('method not allowed', { status: 405 });
  }

  // The origin is a Spin variable (see spin.toml), read at request time
  const origin = Variables.get('origin');
  if (!origin) {
    return new Response('origin variable not set', { status: 500 });
  }

  // Keep the incoming path and query string, e.g. /api/people/1/?format=wookiee
  const incoming = new URL(request.url);
  const url = `${origin}${incoming.pathname}${incoming.search}`;

  // Forward the client's accept header (or accept anything)
  const accept = request.headers.get('accept') ?? '*/*';
  const upstream = await fetch(url, { headers: { accept } });
  const body = await upstream.text();

  // Return the upstream status and body, but only the headers we choose:
  // the upstream content-type plus our own x-intercepted-value header
  return new Response(body, {
    status: upstream.status,
    headers: {
      'content-type': upstream.headers.get('content-type') ?? 'text/plain',
      'x-intercepted-value': 'foobar'
    }
  });
}

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request));
});
