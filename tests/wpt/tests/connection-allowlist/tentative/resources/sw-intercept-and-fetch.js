// Service Worker that intercepts fetches containing 'blank-with-cors.html'
// and performs an outgoing fetch of the same request.
// It records whether it intercepted one and reports that when messaged.

let intercepted = false;

self.addEventListener('fetch', (event) => {
  if (event.request.url.includes('blank-with-cors.html')) {
    intercepted = true;
    event.respondWith(fetch(event.request));
  }
});

self.addEventListener('message', (event) => {
  event.ports[0].postMessage(intercepted);
});
