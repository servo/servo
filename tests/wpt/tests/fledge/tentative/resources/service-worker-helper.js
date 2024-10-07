"use strict;"

// Service workers, once activated, will use 'clients.claim()'
// so that clients loaded in the same scope do not need to be reloaded
// before their fetches will go through this service worker.
// (https://developer.mozilla.org/en-US/docs/Web/API/Clients/claim)
self.addEventListener("activate", (event) => {
  event.waitUntil(clients.claim());
});

// The service worker intercepts fetch calls and posts a message with the url to the
// 'requests-test' broadcast channel, which the test should be listening for.
self.addEventListener('fetch', (event) => {
  const requestChannel = new BroadcastChannel('requests-test');
  var url = event.request.url;

  requestChannel.postMessage({
    url: url,
    message: "Service worker saw this URL: " + url
  });
});