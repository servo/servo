self.addEventListener('fetch', function(event) {
    if (event.request.url.indexOf('dummy.js') != -1) {
      event.respondWith(new Response());
    }
  });
