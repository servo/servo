self.addEventListener('install', () => {
    // activate this service worker immediately
    self.skipWaiting();
});

self.addEventListener('activate', event => {
    // let this service worker control window clients immediately
    event.waitUntil(self.clients.claim());
});
