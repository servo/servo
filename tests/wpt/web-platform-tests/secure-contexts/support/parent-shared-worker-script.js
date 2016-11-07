addEventListener("connect", function (e) {
  var port = e.ports[0];
  port.start();
  var w = new Worker("dedicated-worker-script.js");
  w.onmessage = function (e) {
    port.postMessage(e.data);
  }
});
