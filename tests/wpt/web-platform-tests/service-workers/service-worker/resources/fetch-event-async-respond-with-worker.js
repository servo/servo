var result;

self.addEventListener('message', function(event) {
    event.data.port.postMessage(result);
  });

self.addEventListener('fetch', function(event) {
    setTimeout(function() {
        try {
          event.respondWith(new Response());
          result = 'FAIL: did not throw';
        } catch (error) {
          if (error.name == 'InvalidStateError')
            result = 'PASS';
          else
            result = 'FAIL: Unexpected exception: ' + error;
        }
      }, 0);
  });
