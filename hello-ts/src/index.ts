function handle(_request: Request): Response {
  return new Response('Hello, this is Spin running on Akamai Functions!', {
    status: 200,
    headers: {
      'content-type': 'text/plain'
    }
  });
}

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request));
});
