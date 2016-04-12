self.state = 'starting';

self.addEventListener('install', function() {
    self.state = 'installing';
  });

self.addEventListener('message', function(event) {
    var port = event.data.port;
    if (self.state !== 'installing') {
      port.postMessage('FAIL: Worker should be waiting in installed state');
      return;
    }
    self.skipWaiting()
      .then(function(result) {
          if (result !== undefined) {
            port.postMessage('FAIL: Promise should be resolved with undefined');
            return;
          }
          port.postMessage('PASS');
        })
      .catch(function(e) {
          port.postMessage('FAIL: unexpected exception: ' + e);
        });
  });
