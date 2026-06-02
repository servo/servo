if (typeof ServiceWorkerGlobalScope === "function") {
  self.onmessage = function (e) { e.source.postMessage("ping"); };
} else if (typeof SharedWorkerGlobalScope === "function") {
  onconnect = function (e) {
    var port = e.ports[0];

    port.onmessage = function () { port.postMessage("ping"); }
    port.postMessage("ping");
  };
} else if (typeof DedicatedWorkerGlobalScope === "function") {
  self.postMessage("ping");
}
