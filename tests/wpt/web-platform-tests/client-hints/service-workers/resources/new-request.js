self.addEventListener('fetch', (event) => {
    event.respondWith(fetch("/client-hints/service-workers/resources/echo-hint-in-html.py"))
});
