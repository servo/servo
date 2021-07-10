// For DedicatedWorker and ServiceWorker
self.addEventListener('message', (e) => {
  e.data.port.postMessage(self.crossOriginIsolated);
});

// For SharedWorker
self.addEventListener('connect', (e) => {
  e.ports[0].onmessage = (ev) => {
    ev.data.port.postMessage(self.crossOriginIsolated);
  };
});