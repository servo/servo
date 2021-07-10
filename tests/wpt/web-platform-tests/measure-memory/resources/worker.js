self.onmessage = function(message) {
  const length = message.data.bytes;
  self.root = new Uint8Array(length);
  // Set some elements to disable potential copy-on-write optimizations.
  for (let i = 0; i < length; i += 256) {
    self.root[i] = 1;
  }
  postMessage(self.location.href);
}
