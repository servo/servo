importScripts('get-host-info.sub.js');

var worker_text = 'postMessage("worker loading intercepted by service worker"); ';

self.onfetch = function(event) {
  if (event.request.url.indexOf('synthesized') != -1) {
    event.respondWith(new Response(worker_text));
  } else if (event.request.url.indexOf('same-origin') != -1) {
    event.respondWith(fetch('dummy-worker-script.py'));
  } else if (event.request.url.indexOf('cors') != -1) {
    var path = (new URL('dummy-worker-script.py', self.location)).pathname;
    var url = get_host_info()['HTTPS_REMOTE_ORIGIN'] + path;
    var mode = "no-cors";
    if (event.request.url.indexOf('no-cors') == -1) {
      url += '?ACAOrigin=*';
      mode = "cors";
    }
    event.respondWith(fetch(url, { mode: mode }));
  }
};

