onrtctransform = (event) => {
    const transformer = event.transformer;

    transformer.reader = transformer.readable.getReader();
    transformer.writer = transformer.writable.getWriter();

    let isFirstFrame = true;
    function process(transformer)
    {
        transformer.reader.read().then(chunk => {
            if (chunk.done)
                return;

            if (isFirstFrame) {
                isFirstFrame = false;
                self.postMessage({ name: transformer.options.name, timestamp: chunk.value.timestamp, metadata: chunk.value.getMetadata() });
            }
            transformer.writer.write(chunk.value);
            process(transformer);
        });
    }
    process(transformer);
};
self.postMessage("registered");
