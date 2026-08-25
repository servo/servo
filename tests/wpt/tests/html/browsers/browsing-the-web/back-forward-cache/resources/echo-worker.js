// On receiving a message from the parent Document, send back a message to the
// parent Document. This is used to wait for worker initialization and test
// that this worker is alive and working.

// For dedicated workers.
self.addEventListener('message', event => {
  postMessage(event.data);
});

// For shared workers.
onconnect = e => {
  const port = e.ports[0];
  port.onmessage = event => {
    port.postMessage(event.data);
  }
};
