'use strict';

import {
  recordRequest,
  getRecords,
  resetRecords } from './static-router-sw.sub.js';

// Two caches, created in this order. The first cache never holds the
// requests that are routed to the "cache" source, so a lookup that stops at
// the first cache lacking the request would fall back to the network.
const FIRST_CACHE = 'static-router-multiple-caches-first';
const SECOND_CACHE = 'static-router-multiple-caches-second';

const routerRules = [
  {
    condition: {urlPattern: '/**/multiple-caches-second-only.txt'},
    source: 'cache'
  },
  {
    condition: {urlPattern: '/**/multiple-caches-both.txt'},
    source: 'cache'
  },
  {
    condition: {urlPattern: '/**/multiple-caches-named-first.txt'},
    source: {cacheName: FIRST_CACHE}
  },
  {
    condition: {urlPattern: '/**/multiple-caches-named-second.txt'},
    source: {cacheName: SECOND_CACHE}
  },
  {condition: {urlPattern: '/**/direct.txt'}, source: 'cache'},
];

self.addEventListener('install', e => {
  const addRoutesPromise = e.addRoutes(routerRules);
  e.waitUntil((async () => {
    await addRoutesPromise;
    await caches.delete(FIRST_CACHE);
    await caches.delete(SECOND_CACHE);
    const first = await caches.open(FIRST_CACHE);
    await first.put('unrelated.txt', new Response('Unrelated'));
    await first.put(
        'multiple-caches-both.txt', new Response('From first cache'));
    const second = await caches.open(SECOND_CACHE);
    await second.put(
        'multiple-caches-second-only.txt', new Response('From second cache'));
    await second.put(
        'multiple-caches-both.txt', new Response('From second cache'));
    await second.put(
        'multiple-caches-named-first.txt', new Response('From second cache'));
    await second.put(
        'multiple-caches-named-second.txt', new Response('From second cache'));
  })());
  self.skipWaiting();
});

self.addEventListener('activate', e => {
  e.waitUntil(clients.claim());
});

self.addEventListener('fetch', event => {
  recordRequest(event.request);
  event.respondWith(new Response('From fetch handler'));
});

self.addEventListener('message', event => {
  if (event.data.reset) {
    resetRecords();
  }
  if (event.data.port) {
    const {requests, errors} = getRecords();
    event.data.port.postMessage({requests, errors});
  }
});
