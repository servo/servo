// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-dataview.prototype.byteoffset
description: |
  throws a TypeError if the underlying ArrayBuffer is resized beyond the
  boundary of the dynamically-sized DataView instance
features: [resizable-arraybuffer]
---*/

// If the host chooses to throw as allowed by the specification, the observed
// behavior will be identical to the case where `ArrayBuffer.prototype.resize`
// has not been implemented. The following assertion prevents this test from
// passing in runtimes which have not implemented the method.
assert.sameValue(typeof ArrayBuffer.prototype.resize, "function");

var ab = new ArrayBuffer(4, {maxByteLength: 5});
var dataView = new DataView(ab, 1);

assert.sameValue(dataView.byteOffset, 1);

try {
  ab.resize(5);
} catch (_) {}

assert.sameValue(dataView.byteOffset, 1, "following grow");

try {
  ab.resize(3);
} catch (_) {}

assert.sameValue(dataView.byteOffset, 1, "following shrink (within bounds)");

try {
  ab.resize(1);
} catch (_) {}

assert.sameValue(dataView.byteOffset, 1, "following shrink (on boundary)");

var expectedError;
try {
  ab.resize(0);
  expectedError = TypeError;
} catch (_) {
  expectedError = Test262Error;
}

assert.throws(expectedError, function() {
  dataView.byteOffset;
  throw new Test262Error('the operation completed successfully');
});
