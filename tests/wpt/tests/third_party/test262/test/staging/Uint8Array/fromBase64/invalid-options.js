// Copyright (C) 2025 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-uint8array.frombase64
description: >
  Uint8Array.fromBase64 throws a TypeError when alphabet and
  lastChunkHandling are invalid string values.
includes: [compareArray.js]
features: [uint8array-base64, TypedArray]
---*/

let string = 'SGVsbG8gV29ybGQ=';
assert.compareArray(
    Uint8Array.fromBase64(string),
    [72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100])

// invalid alphabet -----

// shorter length
assert.throws(TypeError, function() {
  Uint8Array.fromBase64(string, {alphabet: 'base'});
});
// same length but invalid value
assert.throws(TypeError, function() {
  Uint8Array.fromBase64(string, {alphabet: 'base65'});
});
// longer length
assert.throws(TypeError, function() {
  Uint8Array.fromBase64(string, {alphabet: 'base64urlurl'});
});
// invalid two-byte value
assert.throws(TypeError, function() {
    Uint8Array.fromBase64(string, {alphabet: '☉‿⊙'});
  });

// invalid lastChunkHandling -----

// shorter length
assert.throws(TypeError, function() {
  Uint8Array.fromBase64(string, {lastChunkHandling: 'stric'});
});
// same length but invalid value
assert.throws(TypeError, function() {
  Uint8Array.fromBase64(string, {lastChunkHandling: 'looss'});
});
// longer length
assert.throws(TypeError, function() {
  Uint8Array.fromBase64(
      string, {lastChunkHandling: 'stop-before-partial-partial'});
});
// invalid two-byte value
assert.throws(TypeError, function() {
    Uint8Array.fromBase64(string, {lastChunkHandling: '☉‿⊙'});
  });
