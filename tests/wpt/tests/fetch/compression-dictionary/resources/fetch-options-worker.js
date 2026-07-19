self.addEventListener('fetch', function(event) {
  if (event.request.url.includes('dummy')) {
    const params = new URL(event.request.url).searchParams;
    const credentials = params.get("credentials");
    const mode = params.get("mode");
    if ((!mode || mode == event.request.mode) &&
        (!credentials || credentials == event.request.credentials)) {
      event.respondWith(fetch(event.request));
    } else {
      event.respondWith(Response.error());
    }
  }
});
