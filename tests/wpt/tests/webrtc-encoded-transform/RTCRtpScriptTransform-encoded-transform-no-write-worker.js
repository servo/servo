onrtctransform = (event) => {
    const transformer = event.transformer;
    transformer.reader = transformer.readable.getReader();

    async function process(transformer)
    {
        const chunk = await transformer.reader.read();
        if (chunk.done)
            return;
        if (transformer.options.name === 'receiver') // receiver
            self.postMessage("received frame.");

        await process(transformer);
    }
    process(transformer);
};
self.postMessage("registered");
