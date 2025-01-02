self.addEventListener('install', e => e.waitUntil(skipWaiting()));
self.addEventListener('activate', e => e.waitUntil(clients.claim()));

self.addEventListener('message', event => {
  try {
    self.registration.paymentManager;
  } catch (e) {
    event.source.postMessage(e);
  }
});
