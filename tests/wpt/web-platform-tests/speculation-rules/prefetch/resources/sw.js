self.addEventListener('fetch', () => event.respondWith(fetch(event.request)));
