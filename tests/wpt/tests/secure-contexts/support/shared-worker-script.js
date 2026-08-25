addEventListener("connect", function (e) {
  var port = e.ports[0];
  port.start();
  port.postMessage(isSecureContext);
});
