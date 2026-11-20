const SWAPI = 'https://swapi.dev/api';

// Helper to send a JSON response
function json(data: unknown, status: number): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'content-type': 'application/json' }
  });
}

// Fetches a person from SWAPI and returns its JSON body unchanged
async function fetchPerson(id: string): Promise<Response> {
  // SWAPI expects a trailing slash, otherwise it redirects
  const upstream = await fetch(`${SWAPI}/people/${id}/`);
  return new Response(await upstream.text(), {
    status: upstream.status,
    headers: { 'content-type': 'application/json' }
  });
}

async function handle(request: Request): Promise<Response> {
  const { pathname } = new URL(request.url);

  if (request.method !== 'GET') {
    return json({ error: 'method not allowed' }, 405);
  }

  // GET / returns Luke Skywalker
  if (pathname === '/') {
    return fetchPerson('1');
  }

  // GET /people/<id> returns the person with that id
  if (pathname.startsWith('/people/')) {
    const id = pathname.slice('/people/'.length);
    if (!/^\d+$/.test(id)) {
      return json({ error: 'id must be a number' }, 400);
    }
    return fetchPerson(id);
  }

  return json({ error: 'not found' }, 404);
}

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request));
});
