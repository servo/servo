// Service worker with 'COEP: require-corp' response header.
// This service worker issues a network request for a resource with or without
// CORP response header.

importScripts("/common/get-host-info.sub.js");

self.addEventListener('message', e => {
  e.waitUntil((async () => {
    let result;
    try {
      let url;
      if (e.data === 'WithCorp') {
        url = get_host_info().HTTPS_REMOTE_ORIGIN +
            '/html/cross-origin-embedder-policy/resources/' +
            'nothing-cross-origin-corp.js';
      } else if (e.data === 'WithoutCorp') {
        url = get_host_info().HTTPS_REMOTE_ORIGIN + '/common/blank.html';
      }
      const response = await fetch(url, { mode: 'no-cors' });
      result = response.type;
    } catch (error) {
      result = `Exception: ${error.name}`;
    } finally {
      e.source.postMessage(result);
    }
  })());
});
