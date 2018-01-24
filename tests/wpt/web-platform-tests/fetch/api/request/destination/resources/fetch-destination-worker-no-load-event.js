self.addEventListener('fetch', function(event) {
    if (event.request.url.includes('dummy')) {
        event.waitUntil(async function() {
            let destination = new URL(event.request.url).searchParams.get("dest");
            let client = await self.clients.get(event.clientId);
            if (event.request.destination == destination) {
                client.postMessage("PASS");
            } else {
                client.postMessage("FAIL");
            }
        }())
    }
    event.respondWith(fetch(event.request));
});


