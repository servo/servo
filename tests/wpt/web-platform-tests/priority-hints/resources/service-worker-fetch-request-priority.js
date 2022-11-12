// This worker echos back the priority of FetchEvent.request.priority

self.addEventListener('message', function(event) {
  self.port = event.data.port;
});

self.addEventListener('fetch', function(event) {
  const search = new URL(event.request.url).search;
  if (search.startsWith('?priority')) {
    try {
      self.port.postMessage(event.request.priority);
    } catch (e) {
      self.port.postMessage('EXCEPTION');
    }
    event.respondWith(new Response(null, {"status": 200}))
  }
});
