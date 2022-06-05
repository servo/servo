self.addEventListener('message', function(event) {
    self.clients.claim()
      .then(function(result) {
          if (result !== undefined) {
              event.data.port.postMessage(
                  'FAIL: claim() should be resolved with undefined');
              return;
          }
          event.data.port.postMessage('PASS');
        })
      .catch(function(error) {
          event.data.port.postMessage('FAIL: exception: ' + error.name);
        });
  });

self.addEventListener('fetch', e => {
    if (e.request.url.match(/\/is-controlled/)) {
      e.respondWith(new Response('controlled'));
    }
    else if (e.request.url.match(/\/get-clients-matchall/)) {
      const options = { includeUncontrolled: true, type: 'all' };
      e.respondWith(
        self.clients.matchAll(options)
          .then(clients => {
            const client_urls = [];
            clients.forEach(client => client_urls.push(client.url));
            return new Response(JSON.stringify(client_urls));
          })
      );
    }
  });
