// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-dataview.prototype.bytelength
description: |
  Returns 0 when the instance is length-tracking and the underlying ArrayBuffer
  shrinks beneath its start offset
info: |
  get DataView.prototype.byteLength

  5. If IsViewOutOfBounds(viewRecord) is true, return +0𝔽.

  IsViewOutOfBounds ( viewRecord )

  5. Let byteOffsetStart be view.[[ByteOffset]].
  6. If view.[[ByteLength]] is auto, then
    a. Let byteOffsetEnd be bufferByteLength.
  ...
  9. If byteOffsetStart > bufferByteLength or byteOffsetEnd > bufferByteLength,
     return true.
  10. Return false.
features: [resizable-arraybuffer]
---*/

// If the host chooses to throw as allowed by the specification, the observed
// behavior will be identical to the case where `ArrayBuffer.prototype.resize`
// has not been implemented. The following assertion prevents this test from
// passing in runtimes which have not implemented the method.
assert.sameValue(typeof ArrayBuffer.prototype.resize, "function");

var ab = new ArrayBuffer(4, {maxByteLength: 5});
var dataView = new DataView(ab, 1);
var expected = 3;

assert.sameValue(dataView.byteLength, expected);

try {
  ab.resize(5);
  expected = 4;
} catch (_) {}

assert.sameValue(dataView.byteLength, expected, "following grow");

try {
  ab.resize(3);
  expected = 2;
} catch (_) {}

assert.sameValue(dataView.byteLength, expected, "following shrink (within bounds)");

try {
  ab.resize(1);
  expected = 0;
} catch (_) {}

assert.sameValue(dataView.byteLength, expected, "following shrink (on boundary)");

try {
  ab.resize(0);
  expected = 0;
} catch (_) {}

assert.sameValue(dataView.byteLength, expected, "following shrink (out of bounds)");

try {
  ab.resize(2);
  expected = 1;
} catch (_) {}

assert.sameValue(dataView.byteLength, expected, "following grow (back into bounds)");
