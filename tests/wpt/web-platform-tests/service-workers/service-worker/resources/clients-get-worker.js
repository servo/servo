self.onfetch = function(e) {
  if (e.request.url.indexOf("clients-get-frame.html") >= 0) {
    if (e.clientId === null) {
      e.respondWith(fetch(e.request));
    } else {
      e.respondWith(Response.error());
    }
    return;
  }
  e.respondWith(new Response(e.clientId));
};

self.onmessage = function(e) {
  var port = e.data.port;
  if (e.data.message == 'get_client_ids') {
    var clientIds = e.data.clientIds;
    var message = [];

    Promise.all(
        clientIds.map(function(clientId) {
          return self.clients.get(clientId);
        }).concat(self.clients.get("invalid-id"))
      ).then(function(clients) {
        clients.forEach(function(client) {
            if (client instanceof Client) {
              message.push([client.visibilityState,
                            client.focused,
                            client.url,
                            client.frameType]);
            } else {
              message.push(client);
            }
          });
        port.postMessage(message);
      });
  } else if (e.data.message == 'get_other_client_id') {
    var clientId = e.data.clientId;
    var message;

    self.clients.get(clientId)
      .then(function(client) {
          if (client instanceof Client) {
            message = [client.visibilityState,
                       client.focused,
                       client.url,
                       client.frameType];
          } else {
            message = client;
          }
          port.postMessage(message);
        });
  }
};
