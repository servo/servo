self.addEventListener('activate', () => self.registration.navigationPreload.enable());
self.addEventListener('fetch', (event) => event.respondWith(event.preloadResponse));
