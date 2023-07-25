onrtctransform = (event) => {
    const transformer = event.transformer;
    transformer.options.port.onmessage = (event) => {
      if (event.data == "ping") {
        transformer.options.port.postMessage("pong");
      }
    };

    transformer.options.port.postMessage("started");
    transformer.reader = transformer.readable.getReader();
    transformer.writer = transformer.writable.getWriter();

    function process(transformer)
    {
        transformer.reader.read().then(chunk => {
            if (chunk.done)
                return;
            if (chunk.value instanceof RTCEncodedVideoFrame) {
                transformer.options.port.postMessage("video chunk");
                if (chunk.value.type == "key") {
                  transformer.options.port.postMessage("video keyframe");
                }
            }
            else if (chunk.value instanceof RTCEncodedAudioFrame)
                transformer.options.port.postMessage("audio chunk");
            transformer.writer.write(chunk.value);
            process(transformer);
        });
    }

    process(transformer);
};
self.postMessage("registered");
