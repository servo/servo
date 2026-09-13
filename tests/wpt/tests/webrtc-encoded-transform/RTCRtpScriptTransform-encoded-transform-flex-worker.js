onrtctransform = (event) => {
  const transformer = event.transformer;
  transformer.reader = transformer.readable.getReader();
  transformer.writer = transformer.writable.getWriter();
  let firstFrame = true;

  async function process(transformer)
  {
    const chunk = await transformer.reader.read();
    if (chunk.done)
      return;
    if (transformer.options.name === "sender") {
      switch (transformer.options.mode) {
        case "inplace":
          new Uint8Array(chunk.value.data).fill(0xa5);
          break;
        case "detach":
          const yoink = chunk.value.data.transfer();
          break;
        case "overwrite":
          const replacement = new ArrayBuffer(16);
          new Uint8Array(replacement).fill(0xa5);
          chunk.value.data = replacement;
          break;
        case "clone-frame":
          const clone = structuredClone(chunk.value);
          new Uint8Array(chunk.value.data).fill(0xa5);
          new Uint8Array(clone.data).fill(0xb5);
          break;
      }
    }

    if (transformer.options.mode != "detach") {
      if (!(new Uint8Array(chunk.value.data).every(b => b == 0xa5))) {
        self.postMessage("Frame was not modified!");
      }
    } else if (!chunk.value.data.detached) {
      self.postMessage("Frame was not detached!");
    }

    const metadataBeforeWrite = JSON.stringify(chunk.value.getMetadata());
    const timestampBeforeWrite = chunk.value.timestamp;

    transformer.writer.write(chunk.value);

    if (!chunk.value.data.detached) {
      self.postMessage("Frame was not detached by write!");
    }

    // Consuming the frame detaches its data, but the metadata is not part of
    // the data and has to survive.
    if (JSON.stringify(chunk.value.getMetadata()) != metadataBeforeWrite) {
      self.postMessage("Frame metadata should survive being consumed!");
    }

    try {
      structuredClone(chunk.value);
      self.postMessage("structuredClone should not work on consumed frame!");
    } catch (e) {
      if (e.name != "DataCloneError") {
        self.postMessage(`expected DataCloneError on consumed frame, got ${e.name}`);
      }
    }

    if (firstFrame) {
      self.postMessage(`${transformer.options.name} written`);
    }
    firstFrame = false;

    await process(transformer);
  }
  process(transformer);
};
self.postMessage("registered");
