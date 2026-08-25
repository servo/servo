self.onmessage = function(e) {
  var port = e.data.port;
  var url = e.data.url;

  e.waitUntil(self.clients.matchAll({includeUncontrolled: true})
    .then(function(clients) {
        var frame_type = "none";
        for (client of clients) {
          if (client.url === url) {
            frame_type = client.frameType;
            break;
          }
        }
        port.postMessage(frame_type);
      })
    .catch(e => {
        port.postMessage('clients.matchAll() rejected: ' + e);
      }));
};