self.addEventListener('install', () => {
    // activate this service worker immediately
    self.skipWaiting();
});

self.addEventListener('activate', event => {
    // let this service worker control window clients immediately
    event.waitUntil(self.clients.claim());
});

self.addEventListener('message', event => {
    event.waitUntil(clients.matchAll().then(windows => {
        event.ports[0].postMessage(windows.map(w => { return w.url; }).sort());
    }));
});
