const requests = new Set();

addEventListener('install', evt => {
  evt.waitUntil(self.skipWaiting());
});

addEventListener('activate', evt => {
  evt.waitUntil(self.clients.claim());
});

addEventListener('message', evt => {
  evt.source.postMessage(requests);
});

addEventListener('fetch', evt => {
  const url = new URL(evt.request.url);
  const path = url.pathname;
  const search = url.search || "?";
  if (path.includes('404')) {
    const dir = path.split('/');
    const request = dir[dir.length-1] + search;
    if (!requests.has(request)) {
      requests.add(request);
    }
    evt.respondWith(new Response(""));
  } else if (path.endsWith('resources.html')) {
    const html = (new URLSearchParams(search)).get('html');
    evt.respondWith(new Response(html, {
      headers: {
        "Content-Type": "text/html"
      }
    }));
  }
  return;
});
