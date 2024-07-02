self.addEventListener('activate', (e) => {
  e.waitUntil(clients.claim());
});

self.addEventListener('message', (e) => {
  e.waitUntil((async () => {

    const url = new URL(e.data.url);
    const request = new Request(url, {mode: e.data.mode});
    const cache = await caches.open('v1');

    let response;
    switch(e.data.source) {
      case "service-worker":
        response = new Response('foo');
        break;

      case "network":
        try {
          response = await fetch(request);
        } catch(error) {
          e.source.postMessage('not-stored');
          return;
        }
        break;
    }

    await cache.put(request, response);
    e.source.postMessage('stored');
  })());
})
