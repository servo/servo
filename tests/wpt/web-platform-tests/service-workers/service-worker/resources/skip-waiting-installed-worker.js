self.state = 'starting';

self.addEventListener('install', function() {
    self.state = 'installing';
  });

self.addEventListener('activate', function() {
    self.state = 'activating';
  });

self.addEventListener('message', function(event) {
    var port = event.data.port;
    if (self.state !== 'installing') {
      port.postMessage('FAIL: Worker should be waiting in installed state');
      return;
    }
    event.waitUntil(self.skipWaiting()
      .then(function(result) {
          if (result !== undefined) {
            port.postMessage('FAIL: Promise should be resolved with undefined');
            return;
          }

          if (self.state === 'activating') {
            port.postMessage(
                'FAIL: Promise should be resolved before worker is activated');
            return;
          }

          port.postMessage('PASS');
        })
      .catch(function(e) {
          port.postMessage('FAIL: unexpected exception: ' + e);
        }));
  });
