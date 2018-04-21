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
        redirect: event.request.redirect,
        credentials: event.request.credentials,
        integrity: event.request.integrity,
        destination: event.request.destination
      });
    event.respondWith(Promise.reject());
  });
