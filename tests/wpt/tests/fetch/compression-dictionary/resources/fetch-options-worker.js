self.addEventListener('fetch', function(event) {
  if (event.request.url.includes('dummy')) {
    const params = new URL(event.request.url).searchParams;
    const credentials = params.get("credentials");
    const destination = params.get('destination');
    const mode = params.get("mode");
    const integrity = params.get('integrity');
    const referrer = params.get('referrer');
    const referrerPolicy = params.get('referrerPolicy');
    if ((credentials === null || credentials == event.request.credentials) &&
        (destination === null || destination == event.request.destination) &&
        (mode === null || mode == event.request.mode) &&
        (integrity === null || integrity == event.request.integrity) &&
        (referrer === null || referrer == event.request.referrer) &&
        (referrerPolicy === null ||
         referrerPolicy == event.request.referrerPolicy)) {
      event.respondWith(fetch(event.request));
    } else {
      event.respondWith(Response.error());
    }
  }
});
