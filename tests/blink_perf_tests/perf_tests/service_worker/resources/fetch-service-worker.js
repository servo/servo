self.addEventListener('install', (event) => {
  event.waitUntil(caches.open('test_cache').then((cache) => {
    return cache.add('/service_worker/resources/data/1K_0.txt');
  }));
});

self.addEventListener('fetch', (event) => {
  event.respondWith(async function() {
    const cachedResponse = await caches.match(event.request);
    if (cachedResponse) {
      return cachedResponse;
    }
    return fetch(event.request);
  }());
});
