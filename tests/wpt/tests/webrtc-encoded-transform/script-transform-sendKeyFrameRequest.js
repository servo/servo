onrtctransform = event => {
  const transformer = event.transformer;
  let gotFrame;

  transformer.options.port.onmessage = event => {
    const {method} = event.data;
    if (method == 'sendKeyFrameRequest') {
      sendKeyFrameRequest();
    } else if (method == 'waitForFrame') {
      waitForFrame();
    }
  }

  async function rejectInMs(timeout) {
    return new Promise((_, reject) => {
      const rejectWithTimeout = () => {
        reject(new DOMException(`Timed out after waiting for ${timeout} ms`,
          'TimeoutError'));
      };
      setTimeout(rejectWithTimeout, timeout);
    });
  }

  async function sendKeyFrameRequest() {
    try {
      await Promise.race([transformer.sendKeyFrameRequest(), rejectInMs(8000)]);;
      transformer.options.port.postMessage('success');
    } catch (e) {
      // TODO: This does not work if we send e.name, why?
      transformer.options.port.postMessage(`failure: ${e.name}`);
    }
  }

  async function waitForFrame() {
    try {
      await Promise.race([new Promise(r => gotFrameCallback = r), rejectInMs(8000)]);
      transformer.options.port.postMessage('got frame');
    } catch (e) {
      // TODO: This does not work if we send e.name, why?
      transformer.options.port.postMessage({result: 'failure', value: `${e.name}`, message: `${e.message}`});
    }
  }

  transformer.options.port.postMessage('started');
  transformer.reader = transformer.readable.getReader();
  transformer.writer = transformer.writable.getWriter();

  function process(transformer)
  {
    transformer.reader.read().then(chunk => {
      if (chunk.done)
        return;
      if (gotFrameCallback) {
        gotFrameCallback();
      }
      transformer.writer.write(chunk.value);
      process(transformer);
    });
  }

  process(transformer);
};
self.postMessage('registered');
