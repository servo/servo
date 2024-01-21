'use strict';

import {routerRules} from './router-rules.js';

var requests = [];

self.addEventListener('install', async e => {
  e.waitUntil(caches.open('v1').then(
      cache => {cache.put('cache.txt', new Response('From cache'))}));

  const params = new URLSearchParams(location.search);
  const key = params.get('key');
  await e.addRoutes(routerRules[key]);
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
