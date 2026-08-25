let frameRtpTimestamp;

// Dropping all but the fifth frame on the sender,
// verifying the receiver only gets that frame.
onrtctransform = (event) => {
    const transformer = event.transformer;

    transformer.reader = transformer.readable.getReader();
    transformer.writer = transformer.writable.getWriter();
    let countRead = 0;

    async function process(transformer)
    {
        const chunk = await transformer.reader.read();
        countRead++;
        if (chunk.done)
            return;
        if (transformer.options.name === 'sender') {
            if(countRead == 5) {
                frameRtpTimestamp = chunk.value.getMetadata().rtpTimestamp;
                transformer.writer.write(chunk.value); // only writing once
            }
        } else  {
            if(chunk.value.getMetadata().rtpTimestamp == frameRtpTimestamp && countRead == 1)
                self.postMessage("got expected");
            else
                self.postMessage("received a different frame");
            return;
        }
        await process(transformer);
    }
    process(transformer);
};
self.postMessage("registered");
