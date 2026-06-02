self.addEventListener('message', function(event) {
    if (event.data.type == "claim") {
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
    } else if (event.data.type == "storeClients") {
      self.clients.matchAll()
        .then(function(result) {
          self.storedClients = result;
          event.data.port.postMessage("PASS");
        });
    } else if (event.data.type == "postMessageToStoredClients") {
      for (let client of self.storedClients) {
        client.postMessage("dummyValue");
      }
      event.data.port.postMessage("PASS");
    } else if (event.data.type == 'storeMessagePort') {
      let isCloseEventFired = false;
      const port = event.ports[0];
      port.start();
      port.onmessage = (event) => {
        if (event.data == 'Confirm the ports can communicate') {
          port.postMessage('Receive message');
        } else if (event.data == 'Ask if the close event was fired') {
          port.postMessage(isCloseEventFired);
        }
      };
      port.onclose = () => {
        isCloseEventFired = true;
      };
    }
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
