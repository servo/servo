// META: global=window,worker,shadowrealm
// META: script=decompression-correct-input.js

'use strict';

promise_test(async t => {
    const ds = new DecompressionStream('deflate');
    const reader = ds.readable.getReader();
    const writer = ds.writable.getWriter();
    const writePromise = writer.write(deflateChunkValue);
    const { done, value } = await reader.read();
    assert_array_equals(Array.from(value), trueChunkValue, "value should match");
}, 'decompressing deflated input should work');

promise_test(async t => {
    const ds = new DecompressionStream('gzip');
    const reader = ds.readable.getReader();
    const writer = ds.writable.getWriter();
    const writePromise = writer.write(gzipChunkValue);
    const { done, value } = await reader.read();
    assert_array_equals(Array.from(value), trueChunkValue, "value should match");
}, 'decompressing gzip input should work');

promise_test(async t => {
    const ds = new DecompressionStream('deflate-raw');
    const reader = ds.readable.getReader();
    const writer = ds.writable.getWriter();
    const writePromise = writer.write(deflateRawChunkValue);
    const { done, value } = await reader.read();
    assert_array_equals(Array.from(value), trueChunkValue, "value should match");
}, 'decompressing deflated (with -raw) input should work');
