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
