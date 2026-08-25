self.addEventListener('activate', (ev) => {
  // claim() to control fetch immediately.
  ev.waitUntil(self.clients.claim());
});

self.addEventListener('fetch', (ev) => {
  ev.waitUntil((async () => {
    const client = await self.clients.get(ev.clientId);
    client.postMessage({ url: ev.request.url });
  })());
  ev.respondWith(fetch(ev.request));
})
