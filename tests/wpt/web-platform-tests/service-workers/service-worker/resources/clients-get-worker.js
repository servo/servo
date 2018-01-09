// This worker is designed to expose information about clients that is only available from Service Worker contexts.
//
// In the case of the `onfetch` handler, it provides the `clientId` property of
// the `event` object. In the case of the `onmessage` handler, it provides the
// Client instance attributes of the requested clients.
self.onfetch = function(e) {
  if (e.request.mode === 'navigate' && e.clientId !== "") {
    e.respondWith(Response.error(
      '`clientId` incorrectly set to non-empty string for request with mode `navigate`'
    ));
    return;
  }

  if (/\/clientId$/.test(e.request.url)) {
    e.respondWith(new Response(e.clientId));
    return;
  }
};

self.onmessage = function(e) {
  var port = e.data.port;
  var client_ids = e.data.clientIds;
  var message = [];

  e.waitUntil(Promise.all(
      client_ids.map(function(client_id) {
          return self.clients.get(client_id);
        }))
      .then(function(clients) {
          // No matching client for a given id or a matched client is off-origin
          // from the service worker.
          if (clients.length == 1 && clients[0] == undefined) {
            port.postMessage(clients[0]);
          } else {
            clients.forEach(function(client) {
                if (client instanceof Client) {
                  message.push([client.visibilityState,
                                client.focused,
                                client.url,
                                client.type,
                                client.frameType]);
                } else {
                  message.push(client);
                }
              });
            port.postMessage(message);
          }
        }));
};
