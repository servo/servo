// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.frombase64
description: Handling of final chunks in Uint8Array.fromBase64
includes: [compareArray.js]
features: [uint8array-base64, TypedArray]
---*/

// padding
assert.compareArray(Uint8Array.fromBase64('ZXhhZg=='), [101, 120, 97, 102]);
assert.compareArray(Uint8Array.fromBase64('ZXhhZg==', { lastChunkHandling: 'loose' }), [101, 120, 97, 102]);
assert.compareArray(Uint8Array.fromBase64('ZXhhZg==', { lastChunkHandling: 'stop-before-partial' }), [101, 120, 97, 102]);
assert.compareArray(Uint8Array.fromBase64('ZXhhZg==', { lastChunkHandling: 'strict' }), [101, 120, 97, 102]);

// no padding
assert.compareArray(Uint8Array.fromBase64('ZXhhZg'), [101, 120, 97, 102]);
assert.compareArray(Uint8Array.fromBase64('ZXhhZg', { lastChunkHandling: 'loose' }), [101, 120, 97, 102]);
assert.compareArray(Uint8Array.fromBase64('ZXhhZg', { lastChunkHandling: 'stop-before-partial' }), [101, 120, 97]);
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('ZXhhZg', { lastChunkHandling: 'strict' });
});

// non-zero padding bits
assert.compareArray(Uint8Array.fromBase64('ZXhhZh=='), [101, 120, 97, 102]);
assert.compareArray(Uint8Array.fromBase64('ZXhhZh==', { lastChunkHandling: 'loose' }), [101, 120, 97, 102]);
assert.compareArray(Uint8Array.fromBase64('ZXhhZh==', { lastChunkHandling: 'stop-before-partial' }), [101, 120, 97, 102]);
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('ZXhhZh==', { lastChunkHandling: 'strict' });
});

// non-zero padding bits, no padding
assert.compareArray(Uint8Array.fromBase64('ZXhhZh'), [101, 120, 97, 102]);
assert.compareArray(Uint8Array.fromBase64('ZXhhZh', { lastChunkHandling: 'loose' }), [101, 120, 97, 102]);
assert.compareArray(Uint8Array.fromBase64('ZXhhZh', { lastChunkHandling: 'stop-before-partial' }), [101, 120, 97]);
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('ZXhhZh', { lastChunkHandling: 'strict' });
});

// partial padding
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('ZXhhZg=');
});
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('ZXhhZg=', { lastChunkHandling: 'loose' });
});
assert.compareArray(Uint8Array.fromBase64('ZXhhZg=', { lastChunkHandling: 'stop-before-partial' }), [101, 120, 97]);
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('ZXhhZg=', { lastChunkHandling: 'strict' });
});

// excess padding
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('ZXhhZg===');
});
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('ZXhhZg===', { lastChunkHandling: 'loose' });
});
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('ZXhhZg===', { lastChunkHandling: 'stop-before-partial' });
});
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('ZXhhZg===', { lastChunkHandling: 'strict' });
});
