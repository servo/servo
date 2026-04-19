// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype.length
description: |
  reset to 0 if the underlying ArrayBuffer is resized beyond the boundary of
  the dynamically-sized TypedArray instance
includes: [testTypedArray.js]
features: [TypedArray, resizable-arraybuffer]
---*/

// If the host chooses to throw as allowed by the specification, the observed
// behavior will be identical to the case where `ArrayBuffer.prototype.resize`
// has not been implemented. The following assertion prevents this test from
// passing in runtimes which have not implemented the method.
assert.sameValue(typeof ArrayBuffer.prototype.resize, "function");

testWithTypedArrayConstructors(function(TA) {
  var BPE = TA.BYTES_PER_ELEMENT;
  var ab = new ArrayBuffer(BPE * 4, {maxByteLength: BPE * 5});
  var array = new TA(ab, BPE);
  var expected = 3;

  assert.sameValue(array.length, expected, "initial value");

  try {
    ab.resize(BPE * 5);
    expected = 4;
  } catch (_) {}

  assert.sameValue(array.length, expected, "following grow");

  try {
    ab.resize(BPE * 3);
    expected = 2;
  } catch (_) {}

  assert.sameValue(array.length, expected, "following shrink (within bounds)");

  try {
    ab.resize(BPE * 3 - 1);
    expected = 1;
  } catch (_) {}

  assert.sameValue(array.length, expected, "following shrink (partial element)");

  try {
    ab.resize(BPE);
    expected = 0;
  } catch (_) {}

  assert.sameValue(array.length, expected, "following shrink (on boundary)");

  try {
    ab.resize(0);
    expected = 0;
  } catch (_) {}

  assert.sameValue(array.length, expected, "following shrink (out of bounds)");
}, null, ["passthrough"]);
