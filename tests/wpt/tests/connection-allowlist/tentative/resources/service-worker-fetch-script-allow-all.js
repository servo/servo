self.addEventListener('activate', (event) => {
  // This enables the document which loads this worker to be immediately
  // controlled by it.
  event.waitUntil(clients.claim());
});

self.addEventListener('fetch', (e) => {
  if (e.request.url.includes('blank-with-cors.html')) {
    e.respondWith(fetch(e.request));
  }
});
