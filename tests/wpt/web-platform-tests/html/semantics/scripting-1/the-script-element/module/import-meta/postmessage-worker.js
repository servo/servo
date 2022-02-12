if ('DedicatedWorkerGlobalScope' in self &&
    self instanceof DedicatedWorkerGlobalScope) {
  postMessage(import.meta.url);
} else if (
    'SharedWorkerGlobalScope' in self &&
    self instanceof SharedWorkerGlobalScope) {
  self.onconnect = function(e) {
    const port = e.ports[0];
    port.start();
    port.postMessage(import.meta.url);
  };
}
