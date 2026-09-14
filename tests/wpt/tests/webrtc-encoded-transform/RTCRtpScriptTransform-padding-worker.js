// A padding-only RTP packet carries no payload, and so is not a frame. It
// should never reach a receive-side transform.
//
// The sender writes one frame and then drops everything else, which starves
// the pacer and gets bandwidth estimation to pad.
onrtctransform = async ({transformer}) => {
  const reader = transformer.readable.getReader();
  const writer = transformer.writable.getWriter();

  if (transformer.options.name === "receiver") {
    let seenFrame = false;
    while (true) {
      const {value, done} = await reader.read();
      if (done) {
        return;
      }
      if (!value.data.byteLength) {
        self.postMessage("empty frame");
      } else if (!seenFrame) {
        // Padding can replay the frame, so only say so once. That leaves
        // anything the test hears after that a failure.
        seenFrame = true;
        self.postMessage("frame");
      }
      writer.write(value);
    }
  }

  // sender
  const first = await reader.read();
  if (first.done) {
    return;
  }
  writer.write(first.value);

  while (!(await reader.read()).done) {
    // Drop subsequent frames
  }
};
self.postMessage("registered");
