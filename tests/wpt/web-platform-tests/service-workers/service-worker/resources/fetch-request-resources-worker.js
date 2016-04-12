var requests = [];
var port = undefined;

self.onmessage = function(e) {
  var message = e.data;
  if ('port' in message) {
    port = message.port;
    port.postMessage({ready: true});
  }
};

self.addEventListener('fetch', function(event) {
    var url = event.request.url;
    if (url.indexOf('dummy?test') == -1) {
      return;
    }
    port.postMessage({
        url: url,
        mode: event.request.mode,
        credentials: event.request.credentials
      });
    event.respondWith(Promise.reject());
  });
