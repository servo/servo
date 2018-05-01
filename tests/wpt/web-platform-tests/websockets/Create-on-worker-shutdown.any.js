// META: script=websocket.sub.js

async_test(t => {
  function workerCode() {
    close();
    var ws = new WebSocket(self.location.origin.replace('http', 'ws'));
    postMessage(ws.readyState == WebSocket.CONNECTING);
  }

  var workerBlob = new Blob([workerCode.toString() + ";workerCode();"], {
    type: "application/javascript"
  });

  var w = new Worker(URL.createObjectURL(workerBlob));
  w.onmessage = function(e) {
    assert_true(e.data, "WebSocket created on worker shutdown.");
    t.done();
  }
}, 'WebSocket created after a worker self.close()');
