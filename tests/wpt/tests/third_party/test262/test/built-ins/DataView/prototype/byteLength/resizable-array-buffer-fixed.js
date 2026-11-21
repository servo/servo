// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-dataview.prototype.bytelength
description: |
  Returns 0 when the underlying ArrayBuffer shrinks beneath its end offset
info: |
  get DataView.prototype.byteLength

  5. If IsViewOutOfBounds(viewRecord) is true, return +0𝔽.

  IsViewOutOfBounds ( viewRecord )

  5. Let byteOffsetStart be view.[[ByteOffset]].
  ...
  7. Else,
    a. Let byteOffsetEnd be byteOffsetStart + view.[[ByteLength]].
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
var dataView = new DataView(ab, 1, 2);

assert.sameValue(dataView.byteLength, 2);

try {
  ab.resize(5);
} catch (_) {}

assert.sameValue(dataView.byteLength, 2, "following grow");

try {
  ab.resize(3);
} catch (_) {}

assert.sameValue(dataView.byteLength, 2, "following shrink (within bounds)");

var expected;
try {
  ab.resize(2);
  expected = 0;
} catch (_) {
  expected = 2;
}

assert.sameValue(dataView.byteLength, expected, "following shrink (out of bounds)");

try {
  ab.resize(3);
  expected = 2;
} catch (_) {}

assert.sameValue(dataView.byteLength, expected, "following grow (back into bounds)");
