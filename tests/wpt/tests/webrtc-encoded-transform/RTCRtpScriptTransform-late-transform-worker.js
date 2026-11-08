// Drops every frame and reports the first one it reads on `options.port`.
onrtctransform = async (event) => {
  const transformer = event.transformer;
  const port = transformer.options.port;
  const reader = transformer.readable.getReader();
  port.postMessage('started');

  let reported = false;
  while (true) {
    const {done} = await reader.read();
    if (done) {
      return;
    }
    if (!reported) {
      reported = true;
      port.postMessage('frame');
    }
  }
};
self.postMessage('registered');
