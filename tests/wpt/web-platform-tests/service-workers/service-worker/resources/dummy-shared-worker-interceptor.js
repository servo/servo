var worker_text = 'onconnect = function(e) { e.ports[0].postMessage("worker loading intercepted by service worker"); };';

self.onfetch = function(event) {
  if (event.request.url.indexOf('dummy-shared-worker.js') != -1)
    event.respondWith(new Response(worker_text));
};

