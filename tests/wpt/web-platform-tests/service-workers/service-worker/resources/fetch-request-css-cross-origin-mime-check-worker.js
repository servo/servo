importScripts('/common/get-host-info.sub.js');
importScripts('test-helpers.sub.js');

self.addEventListener('fetch', function(event) {
    if (event.request.url.indexOf('cross-origin-css.css') != -1) {
      event.respondWith(fetch(
          get_host_info()['HTTPS_REMOTE_ORIGIN'] + base_path() +
          'fetch-request-css-cross-origin-mime-check-cross.css',
          {mode: 'no-cors'}));
    } else if (event.request.url.indexOf('cross-origin-html.css') != -1) {
      event.respondWith(fetch(
          get_host_info()['HTTPS_REMOTE_ORIGIN'] + base_path() +
          'fetch-request-css-cross-origin-mime-check-cross.html',
          {mode: 'no-cors'}));
    } else if (event.request.url.indexOf('synthetic.css') != -1) {
      event.respondWith(new Response("#synthetic { color: blue; }"));
    } else {
      event.respondWith(fetch(event.request));
    }
  });
