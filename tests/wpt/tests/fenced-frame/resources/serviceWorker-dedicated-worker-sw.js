self.addEventListener('fetch', async (e) => {
  if (e.request.url.includes('fenced_frame_dedicated_worker_test')) {
    e.respondWith(new Response('OK'));
    return;
  }

  e.respondWith(fetch(e.request).catch(() => {
    return new Response('not found');
  }));
})

self.addEventListener('install', () => {
  return self.skipWaiting();
});

self.addEventListener('activate', () => {
  return self.clients.claim();
});
