// META: global=window,worker,shadowrealm

'use strict';

const chunkValues = [
  ["deflate", new Uint8Array([120, 156, 75, 173, 40, 72, 77, 46, 73, 77, 81, 200, 47, 45, 41, 40, 45, 1, 0, 48, 173, 6, 36])],
  ["gzip", new Uint8Array([31, 139, 8, 0, 0, 0, 0, 0, 0, 3, 75, 173, 40, 72, 77, 46, 73, 77, 81, 200, 47, 45, 41, 40, 45, 1, 0, 176, 1, 57, 179, 15, 0, 0, 0])],
];

for (const [format, chunkValue] of chunkValues) {
  promise_test(async t => {
    const ds = new DecompressionStream(format);
    const reader = ds.readable.getReader();
    const writer = ds.writable.getWriter();
    const writePromise = writer.write(chunkValue);
    const { value } = await reader.read();
    assert_equals(value.constructor, Uint8Array, "type should match");
    await writePromise;
  }, `decompressing ${format} output should give Uint8Array chunks`);
}
