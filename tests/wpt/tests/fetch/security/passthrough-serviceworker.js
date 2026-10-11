// A pass-through service worker: for requests carrying the sw_passthrough
// marker it re-issues the request itself via fetch(e.request) and returns the
// result. When such a request's redirect chain leaves this origin and returns
// to it (same-origin -> cross-origin -> same-origin), the response the worker
// returns is opaque even though its final URL is same-origin.
self.addEventListener('install', event => {
  event.waitUntil(self.skipWaiting());
});

self.addEventListener('activate', event => {
  event.waitUntil(self.clients.claim());
});

self.addEventListener('fetch', event => {
  const url = new URL(event.request.url);
  if (url.searchParams.has('sw_passthrough')) {
    event.respondWith(fetch(event.request));
  }
});
