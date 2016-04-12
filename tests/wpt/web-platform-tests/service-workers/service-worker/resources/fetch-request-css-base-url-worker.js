importScripts('../resources/get-host-info.sub.js');
importScripts('test-helpers.sub.js');

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
    if (url.indexOf('fetch-request-css-base-url-style.css') != -1) {
      event.respondWith(fetch(
        get_host_info()['HTTPS_REMOTE_ORIGIN'] + base_path() +
        'fetch-request-css-base-url-style.css',
        {mode: 'no-cors'}));
    } else if (url.indexOf('dummy.png') != -1) {
      port.postMessage({
          url: event.request.url,
          referrer: event.request.referrer
        });
    }
  });
