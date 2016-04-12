var result = 'FAIL: did not throw.';

self.addEventListener('message', function(event) {
    event.data.port.postMessage(result);
  });

self.addEventListener('install', function(event) {
    self.installEvent = event;
  });

self.addEventListener('activate', function(event) {
    try {
      self.installEvent.waitUntil(new Promise(function(){}));
    } catch (error) {
      if (error.name == 'InvalidStateError')
        result = 'PASS';
      else
        result = 'FAIL: unexpected exception: ' + error;
    }
  });
