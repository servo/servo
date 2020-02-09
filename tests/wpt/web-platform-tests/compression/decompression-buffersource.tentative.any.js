// META: global=worker

'use strict';

const compressedBytesWithDeflate = [120, 156, 75, 52, 48, 52, 50, 54, 49, 53, 3, 0, 8, 136, 1, 199];
const compressedBytesWithGzip = [31, 139, 8, 0, 0, 0, 0, 0, 0, 3, 75, 52, 48, 52, 2, 0, 216, 252, 63, 136, 4, 0, 0, 0];
// Two chunk values below were chosen to make the length of the compressed
// output be a multiple of 8 bytes.
const deflateExpectedChunkValue = new TextEncoder().encode('a0123456');
const gzipExpectedChunkValue = new TextEncoder().encode('a012');

const bufferSourceChunksForDeflate = [
  {
    name: 'ArrayBuffer',
    value: new Uint8Array(compressedBytesWithDeflate).buffer
  },
  {
    name: 'Int8Array',
    value: new Int8Array(new Uint8Array(compressedBytesWithDeflate).buffer)
  },
  {
    name: 'Uint8Array',
    value: new Uint8Array(new Uint8Array(compressedBytesWithDeflate).buffer)
  },
  {
    name: 'Uint8ClampedArray',
    value: new Uint8ClampedArray(new Uint8Array(compressedBytesWithDeflate).buffer)
  },
  {
    name: 'Int16Array',
    value: new Int16Array(new Uint8Array(compressedBytesWithDeflate).buffer)
  },
  {
    name: 'Uint16Array',
    value: new Uint16Array(new Uint8Array(compressedBytesWithDeflate).buffer)
  },
  {
    name: 'Int32Array',
    value: new Int32Array(new Uint8Array(compressedBytesWithDeflate).buffer)
  },
  {
    name: 'Uint32Array',
    value: new Uint32Array(new Uint8Array(compressedBytesWithDeflate).buffer)
  },
  {
    name: 'Float32Array',
    value: new Float32Array(new Uint8Array(compressedBytesWithDeflate).buffer)
  },
  {
    name: 'Float64Array',
    value: new Float64Array(new Uint8Array(compressedBytesWithDeflate).buffer)
  },
  {
    name: 'DataView',
    value: new DataView(new Uint8Array(compressedBytesWithDeflate).buffer)
  },
];

const bufferSourceChunksForGzip = [
  {
    name: 'ArrayBuffer',
    value: new Uint8Array(compressedBytesWithGzip).buffer
  },
  {
    name: 'Int8Array',
    value: new Int8Array(new Uint8Array(compressedBytesWithGzip).buffer)
  },
  {
    name: 'Uint8Array',
    value: new Uint8Array(new Uint8Array(compressedBytesWithGzip).buffer)
  },
  {
    name: 'Uint8ClambedArray',
    value: new Uint8ClampedArray(new Uint8Array(compressedBytesWithGzip).buffer)
  },
  {
    name: 'Int16Array',
    value: new Int16Array(new Uint8Array(compressedBytesWithGzip).buffer)
  },
  {
    name: 'Uint16Array',
    value: new Uint16Array(new Uint8Array(compressedBytesWithGzip).buffer)
  },
  {
    name: 'Int32Array',
    value: new Int32Array(new Uint8Array(compressedBytesWithGzip).buffer)
  },
  {
    name: 'Uint32Array',
    value: new Uint32Array(new Uint8Array(compressedBytesWithGzip).buffer)
  },
  {
    name: 'Float32Array',
    value: new Float32Array(new Uint8Array(compressedBytesWithGzip).buffer)
  },
  {
    name: 'Float64Array',
    value: new Float64Array(new Uint8Array(compressedBytesWithGzip).buffer)
  },
  {
    name: 'DataView',
    value: new DataView(new Uint8Array(compressedBytesWithGzip).buffer)
  },
];

for (const chunk of bufferSourceChunksForDeflate) {
  promise_test(async t => {
    const ds = new DecompressionStream('deflate');
    const reader = ds.readable.getReader();
    const writer = ds.writable.getWriter();
    const writePromise = writer.write(chunk.value);
    writer.close();
    const { value } = await reader.read();
    assert_array_equals(Array.from(value), deflateExpectedChunkValue, 'value should match');
  }, `chunk of type ${chunk.name} should work for deflate`);
}

for (const chunk of bufferSourceChunksForGzip) {
  promise_test(async t => {
    const ds = new DecompressionStream('gzip');
    const reader = ds.readable.getReader();
    const writer = ds.writable.getWriter();
    const writePromise = writer.write(chunk.value);
    writer.close();
    const { value } = await reader.read();
    assert_array_equals(Array.from(value), gzipExpectedChunkValue, 'value should match');
  }, `chunk of type ${chunk.name} should work for gzip`);
}
