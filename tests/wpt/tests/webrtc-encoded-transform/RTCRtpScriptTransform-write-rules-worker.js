// Exercises the frame-acceptance rules of the writeEncodedData algorithm:
// https://w3c.github.io/webrtc-encoded-transform/#abstract-opdef-writeencodeddata
//
// Acceptable frames get one payload, unacceptable frames get another, and the
// receiver transform reports what it sees. Counting frames does not
// necessarily work here: bandwidth estimation can pad by replaying the last
// packet sent, so an acceptable frame can legitimately show up more than once
// (needs spec clarification, see
// https://github.com/w3c/webrtc-encoded-transform/issues/310)
const kAccept = "600df00d600df00d600df00d600df00d";
const kReject = "badbadbadbadbadbadbadbadbadbadba";

function setPayload(frame, hex) {
  frame.data = Uint8Array.fromHex(hex).buffer;
}

onrtctransform = async ({transformer}) => {
  const reader = transformer.readable.getReader();
  const writer = transformer.writable.getWriter();
  const {name, mode} = transformer.options;

  if (name === "receiver") {
    let accepted = false;
    while (true) {
      const {value, done} = await reader.read();
      if (done) {
        return;
      }
      const hex = new Uint8Array(value.data).toHex();
      if (hex == kAccept) {
        // Padding can replay the acceptable frame, so only say so once. That
        // leaves anything the test sees after that a failure.
        if (!accepted) {
          accepted = true;
          self.postMessage("accepted");
        }
      } else if (hex.length) {
        // Padding could arrive with an empty payload, which is not the fault
        // of the sender transform logic or a bug we're testing here, so filter
        // it out.
        self.postMessage(`unexpected "${hex}"`);
      }
      writer.write(value);
    }
  }

  // sender
  const first = await reader.read();
  if (first.done) {
    return;
  }

  switch (mode) {
    // Baseline: one frame, written once.
    case "control":
      setPayload(first.value, kAccept);
      writer.write(first.value);
      break;

    // "A processor cannot create frames, or move frames between streams."
    case "clone": {
      const clone = structuredClone(first.value);
      setPayload(clone, kReject);
      writer.write(clone);
      setPayload(first.value, kAccept);
      writer.write(first.value);
      break;
    }
    case "constructed": {
      const constructed = new first.value.constructor(first.value);
      setPayload(constructed, kReject);
      writer.write(constructed);
      setPayload(first.value, kAccept);
      writer.write(first.value);
      break;
    }

    // "A processor cannot reorder frames, although it may delay them or drop
    // them."
    case "twice":
      setPayload(first.value, kAccept);
      writer.write(first.value);
      setPayload(first.value, kReject);
      writer.write(first.value);
      break;
    case "reordered": {
      const second = await reader.read();
      if (second.done) {
        return;
      }
      setPayload(second.value, kAccept);
      writer.write(second.value);
      setPayload(first.value, kReject);
      writer.write(first.value);
      break;
    }
  }

  while (!(await reader.read()).done) {
    // Drop the rest to reduce noise.
  }
};
self.postMessage("registered");
