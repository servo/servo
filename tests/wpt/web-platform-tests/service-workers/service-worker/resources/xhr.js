self.addEventListener('activate', function(event) {
    event.waitUntil(clients.claim());
  });
self.addEventListener('message', function(event) {
    event.data.port.postMessage({xhr: !!("XMLHttpRequest" in self)});
  });
