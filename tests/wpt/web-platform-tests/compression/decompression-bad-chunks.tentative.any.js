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
      // See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`
      return new WebAssembly.Memory({ shared:true, initial:1, maximum:1 }).buffer;
    }
  },
  {
    name: 'shared Uint8Array',
    get value() {
      // See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`
      return new Uint8Array(new WebAssembly.Memory({ shared:true, initial:1, maximum:1 }).buffer)
    }
  },
  {
    name: 'invalid deflate bytes',
    value: new Uint8Array([0, 156, 75, 173, 40, 72, 77, 46, 73, 77, 81, 200, 47, 45, 41, 40, 45, 1, 0, 48, 173, 6, 36])
  },
  {
    name: 'invalid gzip bytes',
    value: new Uint8Array([0, 139, 8, 0, 0, 0, 0, 0, 0, 3, 75, 173, 40, 72, 77, 46, 73, 77, 81, 200, 47, 45, 41, 40, 45, 1, 0, 176, 1, 57, 179, 15, 0, 0, 0])
  },
];

for (const chunk of badChunks) {
  promise_test(async t => {
    const ds = new DecompressionStream('gzip');
    const reader = ds.readable.getReader();
    const writer = ds.writable.getWriter();
    const writePromise = writer.write(chunk.value);
    const readPromise = reader.read();
    await promise_rejects_js(t, TypeError, writePromise, 'write should reject');
    await promise_rejects_js(t, TypeError, readPromise, 'read should reject');
  }, `chunk of type ${chunk.name} should error the stream for gzip`);

  promise_test(async t => {
    const ds = new DecompressionStream('deflate');
    const reader = ds.readable.getReader();
    const writer = ds.writable.getWriter();
    const writePromise = writer.write(chunk.value);
    const readPromise = reader.read();
    await promise_rejects_js(t, TypeError, writePromise, 'write should reject');
    await promise_rejects_js(t, TypeError, readPromise, 'read should reject');
  }, `chunk of type ${chunk.name} should error the stream for deflate`);
}
