importScripts('/common/get-host-info.sub.js');
importScripts('test-helpers.sub.js');

self.addEventListener('fetch', event => {
  const url = new URL(event.request.url);
  const target = url.searchParams.get('target');
  const mode = url.searchParams.get('mode') || 'no-cors';

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
    event.respondWith(fetch(target, {mode}));
  }
});
