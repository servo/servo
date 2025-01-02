var worker = new SharedWorker("subsharedworker.js");
worker.port.onmessage = function(e) {
  postMessage(e.data);
}
