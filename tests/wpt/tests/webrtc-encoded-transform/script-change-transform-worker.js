function appendToBuffer(buffer, value) {
    const result = new ArrayBuffer(buffer.byteLength + 1);
    const byteResult = new Uint8Array(result);
    byteResult.set(new Uint8Array(buffer), 0);
    byteResult[buffer.byteLength] = value;
    return result;
}

onrtctransform = (event) => {
    const transformer = event.transformer;

    transformer.reader = transformer.readable.getReader();
    transformer.writer = transformer.writable.getWriter();

    function process(transformer)
    {
        transformer.reader.read().then(chunk => {
            if (chunk.done)
                return;
            if (transformer.options.name === 'sender1')
                chunk.value.data = appendToBuffer(chunk.value.data, 1);
            else if (transformer.options.name === 'sender2')
                chunk.value.data = appendToBuffer(chunk.value.data, 2);
            else {
                const value = new Uint8Array(chunk.value.data)[chunk.value.data.byteLength - 1];
                if (value !== 1 && value !== 2)
                    self.postMessage("unexpected value: " + value);
                else if (value === 2)
                    self.postMessage("got value 2");
                chunk.value.data = chunk.value.data.slice(0, chunk.value.data.byteLength - 1);
            }
            transformer.writer.write(chunk.value);
            process(transformer);
        });
    }

    process(transformer);
};
self.postMessage("registered");
