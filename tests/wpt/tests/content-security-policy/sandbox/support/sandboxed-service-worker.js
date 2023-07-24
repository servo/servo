self.addEventListener('fetch', function(event) {
    const url = new URL(event.request.url);
    if (url.pathname.indexOf('get-origin') != -1) {
      event.respondWith(new Promise(function(resolve) {
        resolve(new Response(JSON.stringify({
            origin: self.origin
          })));
        }));
    }
    else if (url.pathname.indexOf('fetch') != -1) {
      event.respondWith(fetch(url.searchParams.get('url'),
                              {mode: event.request.mode}));
    }
  });
