importScripts('/common/get-host-info.sub.js');

var remoteUrl = get_host_info()['HTTPS_REMOTE_ORIGIN'] +
  '/service-workers/service-worker/resources/simple.txt'

self.addEventListener('fetch', event => {
    if (!event.request.url.match(/opaque-response\?from=/)) {
      return;
    }

    event.respondWith(fetch(remoteUrl, {mode: 'no-cors'}));
  });
