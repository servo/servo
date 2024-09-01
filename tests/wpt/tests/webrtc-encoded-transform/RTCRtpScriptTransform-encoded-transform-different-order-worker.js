let secondFrame;
let secondTimestamp;

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
            if(countRead == 1){
                transformer.writer.write(chunk.value);
            } else if(countRead == 2){
                secondFrame = chunk.value;
                secondTimestamp = chunk.value.getMetadata().rtpTimestamp;
            } else if(countRead == 3){
                // Write third frame before second frame.
                transformer.writer.write(chunk.value);
                transformer.writer.write(secondFrame);
            } else if(countRead <= 10){
                transformer.writer.write(chunk.value);
            }
        } else  { // receiver
            if (chunk.value.getMetadata().rtpTimestamp == secondTimestamp) {
                self.postMessage("received an unexpected frame");
                return;
            } else if(countRead == 9){
                self.postMessage("got expected");
                return;
            }
            transformer.writer.write(chunk.value);
        }
        await process(transformer);
    }
    process(transformer);
};
self.postMessage("registered");