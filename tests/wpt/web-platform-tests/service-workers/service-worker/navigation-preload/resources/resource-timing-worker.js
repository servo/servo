self.addEventListener('activate', event => {
    event.waitUntil(self.registration.navigationPreload.enable());
  });

self.addEventListener('fetch', event => {
    event.respondWith(
      event.preloadResponse
          .then(response => {
            var headers = response.headers;
            return response.text().then(text =>
              new Response(
                JSON.stringify({
                  decodedBodySize: headers.get('X-Decoded-Body-Size'),
                  encodedBodySize: headers.get('X-Encoded-Body-Size'),
                  timingEntries: performance.getEntriesByName(event.request.url)
                }),
                {headers: {'Content-Type': 'text/html'}}));
          }));
  });
