importScripts('/common/get-host-info.sub.js');
importScripts('test-helpers.sub.js');

self.addEventListener('fetch', event => {
  const url = new URL(event.request.url);
  const target = url.searchParams.get('target');
  const mode = url.searchParams.get('mode') || 'no-cors';
  const use_cache = url.searchParams.get('use_cache') === '1';

  if (target === 'synthetic') {
    event.respondWith(new Response('synthetic body', {
      headers: {
        'Content-Type': 'text/plain',
        'Server-Timing': 'metric;dur=123.4;desc="synthetic"'
      }
    }));
    return;
  }

  if (target) {
    if (use_cache) {
      event.respondWith((async () => {
        const cache = await caches.open('test-cache');
        const response = await fetch(target, {mode});
        await cache.put(event.request, response);
        return await cache.match(event.request);
      })());
    } else {
      event.respondWith(fetch(target, {mode}));
    }
  }
});
