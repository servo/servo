self.addEventListener('fetch', (e) => {
  const url = new URL(e.request.url);
  if (url.pathname.match('controlled-endpoint.py')) {
    e.respondWith(new Response('FROM_SERVICE_WORKER'));
  }
});