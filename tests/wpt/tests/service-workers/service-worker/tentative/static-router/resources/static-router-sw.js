'use strict';

self.addEventListener('install', e => {
  e.registerRouter({
    condition: {urlPattern: "*.txt"},
    source: "network"
  });
  self.skipWaiting();
});

self.addEventListener('activate', e => {
  e.waitUntil(clients.claim());
});

self.addEventListener('fetch', function(event) {
  const url = new URL(event.request.url);
  const nonce = url.searchParams.get('nonce');
  event.respondWith(new Response(nonce));
});
