self.onmessage = function(e) {
  self.clients.matchAll().then(function(clients) {
      clients.forEach(function(client) {
          var messageChannel = new MessageChannel();
          messageChannel.port1.onmessage =
            onMessageViaMessagePort.bind(null, client);
          client.postMessage({port: messageChannel.port2},
                             [messageChannel.port2]);
        });
    });
};

function onMessageViaMessagePort(client, e) {
  var message = e.data;
  if ('value' in message) {
    client.postMessage({ack: 'Acking value: ' + message.value});
  } else if ('done' in message) {
    client.postMessage({done: true});
  }
}
