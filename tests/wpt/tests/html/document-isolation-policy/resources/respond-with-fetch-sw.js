// Responds to requests whose URL has a respond-with-fetch query parameter with a fetch of the request,
// to those whose URL has a respond-with-fetch-of query parameter with a no-cors fetch of its value,
// and to those whose URL has a respond-with-cache-of query parameter with a no-cors fetch of its value
// that it stores in a cache for the request and matches from there.
self.addEventListener("install", () => self.skipWaiting());

self.addEventListener("fetch", event => {
  const params = new URL(event.request.url).searchParams;
  if (params.has("respond-with-fetch"))
    event.respondWith(fetch(event.request));
  else if (params.has("respond-with-fetch-of"))
    event.respondWith(fetch(params.get("respond-with-fetch-of"), { mode: "no-cors" }));
  else if (params.has("respond-with-cache-of")) {
    event.respondWith((async () => {
      const cache = await caches.open("respond-with-cache-of");
      await cache.put(event.request, await fetch(params.get("respond-with-cache-of"), { mode: "no-cors" }));
      return cache.match(event.request);
    })());
  }
});
