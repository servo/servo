// Reports whether a message or a messageerror event was received.
if ('onconnect' in self) {
  self.onconnect = e => {
    const port = e.ports[0];
    port.onmessage = () => port.postMessage('message');
    port.onmessageerror = () => port.postMessage('messageerror');
  };
} else {
  self.onmessage = e => e.source.postMessage('message');
  self.onmessageerror = e => e.source.postMessage('messageerror');
}
