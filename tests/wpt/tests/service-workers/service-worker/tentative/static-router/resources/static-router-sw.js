'use strict';

var requests = [];

self.addEventListener('install', e => {
  e.registerRouter([
    {condition: {urlPattern: '*.txt'}, source: 'network'}, {
      condition: {urlPattern: '*/simple-test-for-condition-main-resource.html'},
      source: 'network'
    }
  ]);
  self.skipWaiting();
});

self.addEventListener('activate', e => {
  e.waitUntil(clients.claim());
});

self.addEventListener('fetch', function(event) {
  requests.push({url: event.request.url, mode: event.request.mode});
  const url = new URL(event.request.url);
  const nonce = url.searchParams.get('nonce');
  event.respondWith(new Response(nonce));
});

self.addEventListener('message', function(event) {
  event.data.port.postMessage({requests: requests});
  requests = [];
});
