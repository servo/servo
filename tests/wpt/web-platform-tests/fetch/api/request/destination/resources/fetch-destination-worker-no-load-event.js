self.addEventListener('fetch', function(event) {
    if (event.request.url.includes('dummy')) {
        event.waitUntil(async function() {
            let destination = new URL(event.request.url).searchParams.get("dest");
            var result = "FAIL";
            if (event.request.destination == destination) {
              result = "PASS";
            }
            let cl = await clients.matchAll({includeUncontrolled: true});
            for (i = 0; i < cl.length; i++) {
              cl[i].postMessage(result);
            }
        }())
    }
    event.respondWith(fetch(event.request));
});


