self.onmessage = function(e) {
  var port = e.data.port;
  var options = e.data.options;

  self.clients.matchAll(options).then(function(clients) {
      var message = [];
      clients.forEach(function(client) {
          var frame_type = client.frameType;
          if (client.url.indexOf('clients-matchall-include-uncontrolled.https.html') > -1 &&
              client.frameType == 'auxiliary') {
            // The test tab might be opened using window.open() by the test framework.
            // In that case, just pretend it's top-level!
            frame_type = 'top-level';
          }
          message.push([client.visibilityState,
                        client.focused,
                        client.url,
                        frame_type]);
        });
      // Sort by url
      message.sort(function(a, b) { return a[2] > b[2] ? 1 : -1; });
      port.postMessage(message);
    });
};
