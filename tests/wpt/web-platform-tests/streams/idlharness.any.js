// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

idl_test(
  ['streams'],
  ['dom'], // for AbortSignal
  async idl_array => {
    // Empty try/catches ensure that if something isn't implemented (e.g., readable byte streams, or writable streams)
    // the harness still sets things up correctly. Note that the corresponding interface tests will still fail.

    try {
      new ReadableStream({
        start(c) {
          self.readableStreamDefaultController = c;
        }
      });
    } catch {}

    try {
      new ReadableStream({
        start(c) {
          self.readableByteStreamController = c;
        },
        type: 'bytes'
      });
    } catch {}

    try {
      const stream = new ReadableStream({
        pull() {
          self.readableStreamByobRequest = controller.byobRequest;
        },
        type: 'bytes'
      });

      const reader = stream.getReader({ mode: 'byob' });

      await reader.read(new Uint8Array(1));
    } catch {}

    try {
      new WritableStream({
        start(c) {
          self.writableStreamDefaultController = c;
        }
      });
    } catch {}

    try {
      new TransformStream({
        start(c) {
          self.transformStreamDefaultController = c;
        }
      });
    } catch {}

    idl_array.add_objects({
      ReadableStream: ["new ReadableStream()"],
      ReadableStreamDefaultReader: ["(new ReadableStream()).getReader()"],
      ReadableStreamBYOBReader: ["(new ReadableStream({ type: 'bytes' })).getReader('byob')"],
      ReadableStreamDefaultController: ["self.readableStreamDefaultController"],
      ReadableByteStreamController: ["self.readableByteStreamController"],
      ReadableStreamBYOBRequest: ["self.readableStreamByobRequest"],
      WritableStream: ["new WritableStream()"],
      WritableStreamDefaultWriter: ["(new WritableStream()).getWriter()"],
      WritableStreamDefaultController: ["self.writableStreamDefaultController"],
      TransformStream: ["new TransformStream()"],
      TransformStreamDefaultController: ["self.transformStreamDefaultController"],
      ByteLengthQueuingStrategy: ["new ByteLengthQueuingStrategy({ highWaterMark: 5 })"],
      CountQueuingStrategy: ["new CountQueuingStrategy({ highWaterMark: 5 })"]
    });
  }
);
