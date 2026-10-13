// Synthesizes a document at any path under this scope ending in "/probe.html",
// so that a test can observe which cookies match a path that has no
// corresponding file. Anything else is left to the network.
//
// Note a service worker cannot inspect the Cookie header of a request, as it is
// a forbidden header name and cookies are attached after the fetch event, so the
// synthesized document reads document.cookie instead.
self.addEventListener('install', event => event.waitUntil(self.skipWaiting()));
self.addEventListener('activate', event => event.waitUntil(self.clients.claim()));

self.addEventListener('fetch', event => {
  const url = new URL(event.request.url);
  if (!url.pathname.endsWith('/probe.html')) {
    return;
  }
  event.respondWith(new Response(
      `<!doctype html><meta charset=utf-8><script>
         parent.postMessage(
             {path: location.pathname, cookie: document.cookie}, "*");
       </script>`,
      {headers: {'Content-Type': 'text/html; charset=utf-8'}}));
});
