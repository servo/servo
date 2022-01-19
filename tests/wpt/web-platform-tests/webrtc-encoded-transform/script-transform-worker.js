onrtctransform = (event) => {
    const transformer = event.transformer;
    transformer.options.port.onmessage = (event) => transformer.options.port.postMessage(event.data);

    self.postMessage("started");
    transformer.reader = transformer.readable.getReader();
    transformer.writer = transformer.writable.getWriter();

    function process(transformer)
    {
        transformer.reader.read().then(chunk => {
            if (chunk.done)
                return;
            if (chunk.value instanceof RTCEncodedVideoFrame)
                self.postMessage("video chunk");
            else if (chunk.value instanceof RTCEncodedAudioFrame)
                self.postMessage("audio chunk");
            transformer.writer.write(chunk.value);
            process(transformer);
        });
    }

    process(transformer);
};
self.postMessage("registered");
