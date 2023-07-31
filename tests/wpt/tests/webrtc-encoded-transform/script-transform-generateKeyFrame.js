onrtctransform = event => {
  const transformer = event.transformer;
  let keyFrameCount = 0;
  let gotFrame;

  transformer.options.port.onmessage = event => {
    const {method, rid} = event.data;
    // Maybe refactor to have transaction ids?
    if (method == 'generateKeyFrame') {
      generateKeyFrame(rid);
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

  async function generateKeyFrame(rid) {
    try {
      const timestamp = await Promise.race([transformer.generateKeyFrame(rid), rejectInMs(8000)]);
      transformer.options.port.postMessage({result: 'success', value: timestamp, count: keyFrameCount});
    } catch (e) {
      // TODO: This does not work if we send e.name, why?
      transformer.options.port.postMessage({result: 'failure', value: `${e.name}`, message: `${e.message}`});
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
      if (chunk.value instanceof RTCEncodedVideoFrame) {
        if (chunk.value.type == 'key') {
          keyFrameCount++;
        }
      }
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
