// Service Worker that intercepts fetches containing 'blank-with-cors.html'
// and performs an outgoing fetch of the same request.
// It also posts a message to controlled window clients upon interception.

self.addEventListener('fetch', (event) => {
  if (event.request.url.includes('blank-with-cors.html')) {
    event.waitUntil(
      self.clients.matchAll({ type: 'window' }).then((clients) => {
        for (const client of clients) {
          client.postMessage({ type: 'sw-intercepted', url: event.request.url });
        }
      })
    );
    event.respondWith(fetch(event.request));
  }
});
