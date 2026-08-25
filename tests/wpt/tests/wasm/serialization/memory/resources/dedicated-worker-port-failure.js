// Lives inside a SharedWorker (sharedworker-spawn-dedicated.js).
// Receives a MessagePort, listens for messages on it, and reports back.
self.onmessage = e => {
  const port = e.ports[0];
  port.onmessage = () => {
    self.postMessage("unexpected-message");
  };
  port.onmessageerror = () => {
    self.postMessage("got-messageerror");
  };
  self.postMessage("ready");
};
