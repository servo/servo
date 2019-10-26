// META: global=worker

'use strict';

const badChunks = [
  {
    name: 'undefined',
    value: undefined
  },
  {
    name: 'null',
    value: null
  },
  {
    name: 'numeric',
    value: 3.14
  },
  {
    name: 'object, not BufferSource',
    value: {}
  },
  {
    name: 'array',
    value: [65]
  },
  {
    name: 'SharedArrayBuffer',
    // Use a getter to postpone construction so that all tests don't fail where
    // SharedArrayBuffer is not yet implemented.
    get value() {
      return new SharedArrayBuffer();
    }
  },
  {
    name: 'shared Uint8Array',
    get value() {
      return new Uint8Array(new SharedArrayBuffer())
    }
  },
];

for (const chunk of badChunks) {
  promise_test(async t => {
    const cs = new CompressionStream('gzip');
    const reader = cs.readable.getReader();
    const writer = cs.writable.getWriter();
    const writePromise = writer.write(chunk.value);
    const readPromise = reader.read();
    await promise_rejects(t, new TypeError(), writePromise, 'write should reject');
    await promise_rejects(t, new TypeError(), readPromise, 'read should reject');
  }, `chunk of type ${chunk.name} should error the stream for gzip`);

  promise_test(async t => {
    const cs = new CompressionStream('deflate');
    const reader = cs.readable.getReader();
    const writer = cs.writable.getWriter();
    const writePromise = writer.write(chunk.value);
    const readPromise = reader.read();
    await promise_rejects(t, new TypeError(), writePromise, 'write should reject');
    await promise_rejects(t, new TypeError(), readPromise, 'read should reject');
  }, `chunk of type ${chunk.name} should error the stream for deflate`);
}
