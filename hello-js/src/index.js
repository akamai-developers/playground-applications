function handle(_request) {
  return new Response('Hello, this is Spin running on Akamai Functions!', {
    status: 200,
    headers: {
      'content-type': 'text/plain'
    }
  });
}

addEventListener('fetch', (event) => {
  event.respondWith(handle(event.request));
});
