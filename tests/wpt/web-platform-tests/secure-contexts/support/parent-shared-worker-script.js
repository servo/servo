addEventListener("connect", function (e) {
  var port = e.ports[0];
  port.start();
  // If nested workers aren't supported, punt:
  if (typeof Worker != "undefined") {
    var w = new Worker("dedicated-worker-script.js");
    w.onmessage = function (e) {
      port.postMessage(e.data);
    }
  } else {
    port.postMessage("Nested workers not supported.");
  }
});
