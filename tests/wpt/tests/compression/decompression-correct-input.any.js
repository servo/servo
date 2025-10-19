// META: global=window,worker,shadowrealm
// META: script=decompression-correct-input.js

'use strict';

const tests = [
  ["deflate", deflateChunkValue],
  ["gzip", gzipChunkValue],
  ["deflate-raw", deflateRawChunkValue],
];

for (const [format, chunk] of tests) {
  promise_test(async t => {
    const ds = new DecompressionStream(format);
    const reader = ds.readable.getReader();
    const writer = ds.writable.getWriter();
    const writePromise = writer.write(chunk);
    const { done, value } = await reader.read();
    assert_array_equals(Array.from(value), trueChunkValue, "value should match");
  }, `decompressing ${format} input should work`);
}
