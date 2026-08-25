var id = 0;
try {
  id = setTimeout("postMessage('handler invoked')", 100);
} catch (e) {}
var message = id === 0 ? "setTimeout blocked" : "setTimeout allowed";

if (typeof SharedWorkerGlobalScope === "function") {
  onconnect = function (e) {
    var port = e.ports[0];

    port.onmessage = function () { port.postMessage(message); };
    port.postMessage(message);
  };
} else if (typeof DedicatedWorkerGlobalScope === "function") {
  self.postMessage(message);
}
