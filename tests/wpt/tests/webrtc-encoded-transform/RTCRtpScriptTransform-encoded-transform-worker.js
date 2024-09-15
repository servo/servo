const modification = 1;

function appendToBuffer(buffer, value) {
    const result = new ArrayBuffer(buffer.byteLength + 1);
    const byteResult = new Uint8Array(result);
    byteResult.set(new Uint8Array(buffer), 0);
    byteResult[buffer.byteLength] = value;
    return result;
}

function ModifyAndWrite(chunk, transformer) {
    chunk.value.data = appendToBuffer(chunk.value.data, modification);
    transformer.writer.write(chunk.value);
}

function RestoreAndWrite(chunk, transformer) {
    const lastByte = new Uint8Array(chunk.value.data)[chunk.value.data.byteLength - 1];
    chunk.value.data = chunk.value.data.slice(0, chunk.value.data.byteLength - 1);
    let frameData = chunk.value.data;
    transformer.writer.write(chunk.value);
    if (lastByte === modification && !chunk.value.getMetadata().rtpTimestamp && frameData.byteLength == 0) {
        self.postMessage("got expected");
    }
    else {
        self.postMessage("unexpected value: lastByte (got " + lastByte +
            ", expected " + modification +
            "),  frame data length (got " + frameData.byteLength +
            ", expected 0), rtpTimestamp (got " +
             chunk.value.getMetadata().rtpTimestamp + ", expected null)");
    }
}
onrtctransform = (event) => {
    const transformer = event.transformer;

    transformer.reader = transformer.readable.getReader();
    transformer.writer = transformer.writable.getWriter();
    async function process(transformer)
    {
        const chunk = await transformer.reader.read();
        if (chunk.done)
            return;
        if (transformer.options.name === 'sender') // sender
            ModifyAndWrite(chunk, transformer);
        else  // receiver
            RestoreAndWrite(chunk, transformer);

        await process(transformer);
    }
    process(transformer);
};
self.postMessage("registered");
