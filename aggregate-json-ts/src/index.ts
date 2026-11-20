const SWAPI = 'https://swapi.dev/api';

// Both SWAPI resources we use have a name field
interface Named {
  name: string;
}

// Helper to send a JSON response
function json(data: unknown, status: number): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'content-type': 'application/json' }
  });
}

// Fetches a SWAPI resource and returns its name; throws on any non-2xx status
async function fetchName(url: string): Promise<string> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`${url} returned ${response.status}`);
  }
  const body = (await response.json()) as Named;
  return body.name;
}

async function handle(request: Request): Promise<Response> {
  const { pathname } = new URL(request.url);

  if (request.method !== 'GET') {
    return json({ error: 'method not allowed' }, 405);
  }

  if (pathname !== '/') {
    return json({ error: 'not found' }, 404);
  }

  try {
    // Send both requests at the same time and wait for both to finish
    const [person, planet] = await Promise.all([
      fetchName(`${SWAPI}/people/1/`),
      fetchName(`${SWAPI}/planets/1/`)
    ]);
    return json({ person, planet }, 200);
  } catch (err) {
    console.error(err);
    return json({ error: 'upstream request failed' }, 502);
  }
}

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request));
});
