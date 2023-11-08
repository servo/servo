'use strict';

var requests = [];

self.addEventListener('install', e => {
  e.registerRouter([
    {condition: {requestMode: 'no-cors'}, source: 'network'}, {
      condition: {urlPattern: '/**/*.txt??*'},
      // Note: "??*" is for allowing arbitrary query strings.
      // Upon my experiment, the URLPattern needs two '?'s for specifying
      // a coming string as a query.
      source: 'network'
    },
    {
      condition:
          {urlPattern: '/**/simple-test-for-condition-main-resource.html'},
      source: 'network'
    },
    {
      condition: {
        or: [
          {
            or: [{urlPattern: '/**/or-test/direct1.*??*'}],
          },
          {urlPattern: '/**/or-test/direct2.*??*'}
        ]
      },
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
