importScripts("helper.js");

const modification = 1;

let frameSender1;
let isWaitingForFirstFrameSender1 = true;
let isWaitingForFirstFrameSender2 = true;
let frameSender2RtpTimestamp;

onrtctransform = (event) => {
    const transformer = event.transformer;

    transformer.reader = transformer.readable.getReader();
    transformer.writer = transformer.writable.getWriter();
    async function process(transformer)
    {
        const chunk = await transformer.reader.read();
        if (chunk.done)
            return;
        if (transformer.options.name === 'sender1' && isWaitingForFirstFrameSender1) {
            isWaitingForFirstFrameSender1 = false;
            frameSender1 = chunk.value;
        } else if(transformer.options.name == "sender2" && isWaitingForFirstFrameSender2){
            isWaitingForFirstFrameSender2 = false;
            transformer.writer.write(frameSender1);
            frameSender2RtpTimestamp = chunk.value.getMetadata().rtpTimestamp;
            chunk.value.data = appendToBuffer(chunk.value.data, modification);
            transformer.writer.write(chunk.value);
        } else if(transformer.options.name == "receiver2") {
            const lastByte =
                new Uint8Array(chunk.value.data)[chunk.value.data.byteLength - 1];
            if (lastByte === modification && frameSender2RtpTimestamp == chunk.value.getMetadata().rtpTimestamp) {
                self.postMessage("got expected");
                return;
            } else {
                self.postMessage("unexpected value of lastByte: got " + lastByte +
                    ", expected " + modification);
                return;
            }
        }

        await process(transformer);
    }
    process(transformer);
};
self.postMessage("registered");
