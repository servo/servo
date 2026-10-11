// Reports whether a message or a messageerror event was received.
if ('onconnect' in self) {
  self.onconnect = e => {
    const port = e.ports[0];
    port.onmessage = () => port.postMessage('message');
    port.onmessageerror = () => port.postMessage('messageerror');
  };
} else if ('clients' in self) {
  self.onmessage = e => e.source.postMessage('message');
  self.onmessageerror = e => e.source.postMessage('messageerror');
} else {
  self.onmessage = () => postMessage('message');
  self.onmessageerror = () => postMessage('messageerror');
}
