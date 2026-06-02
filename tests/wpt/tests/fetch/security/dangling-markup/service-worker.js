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
  const params = new URLSearchParams(search);
  const type = params.get('type');
  if (path.includes('404')) {
    const dir = path.split('/');
    const request = dir[dir.length-1] + search;
    if (!requests.has(request)) {
      requests.add(request);
    }
    evt.respondWith(new Response("", {
      headers: {
        "Content-Type": type || "text/plain"
      }
    }));
  } else if (path.endsWith('resources.html')) {
    const html = params.get('html') || "";
    evt.respondWith(new Response(html, {
      headers: {
        "Content-Type": type || "text/html"
      }
    }));
  }
  return;
});
