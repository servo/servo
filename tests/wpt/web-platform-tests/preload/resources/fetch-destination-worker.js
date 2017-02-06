self.addEventListener('fetch', function(event) {
    if (event.request.url.indexOf('dummy.xml') != -1) {
        if (!event.request.destination || event.request.destination == "")
            event.respondWith(new Response());
        else
            event.respondWith(Response.error());
    }
});

