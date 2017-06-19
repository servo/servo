function match_query(query_string) {
  return self.location.search.substr(1) == query_string;
}

function navigate_test(e) {
  var port = e.data.port;
  var url = e.data.url;

  return clients.matchAll({ includeUncontrolled : true })
    .then(function(client_list) {
        for (var i = 0; i < client_list.length; i++) {
          var client = client_list[i];
          if (client.frameType == 'nested') {
            return client.navigate(url);
          }
        }
        port.postMessage('Could not locate window client.');
      })
    .then(function(new_client) {
        if (new_client === null)
          port.postMessage(new_client);
        else
          port.postMessage(new_client.url);
      })
    .catch(function(error) {
        port.postMessage(error.name);
      });
}

function getTestClient() {
  return clients.matchAll({ includeUncontrolled: true })
    .then(function(client_list) {
        for (var i = 0; i < client_list.length; i++) {
          var client = client_list[i];

          if (/windowclient-navigate\.https\.html/.test(client.url)) {
            return client;
          }
        }

        throw new Error('Service worker was unable to locate test client.');
      });
}

function waitForMessage(client) {
  var channel = new MessageChannel();
  client.postMessage({ port: channel.port2 }, [channel.port2]);

  return new Promise(function(resolve) {
        channel.port1.onmessage = resolve;
      });
}

// The worker must remain in the "installing" state for the duration of some
// sub-tests. In order to achieve this coordination without relying on global
// state, the worker must create a message channel with the client from within
// the "install" event handler.
if (match_query('installing')) {
  self.addEventListener('install', function(e) {
      e.waitUntil(getTestClient().then(waitForMessage));
    });
}

self.addEventListener('message', function(e) {
    e.waitUntil(navigate_test(e));
  });
