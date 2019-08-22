self.addEventListener('activate', (e) => {
  e.waitUntil(clients.claim());
});

self.addEventListener('fetch', (e) => {
  const url = new URL(e.request.url);
  if (url.searchParams.has('passthrough')) {
    return;
  }

  e.respondWith(fetch(e.request));
});
