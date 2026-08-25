// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.map
description: Return abrupt when "this" value fails buffer boundary checks
includes: [testTypedArray.js]
features: [ArrayBuffer, BigInt, TypedArray, arrow-function, resizable-arraybuffer]
---*/

assert.sameValue(
  typeof TypedArray.prototype.map,
  'function',
  'implements TypedArray.prototype.map'
);

assert.sameValue(
  typeof ArrayBuffer.prototype.resize,
  'function',
  'implements ArrayBuffer.prototype.resize'
);

testWithBigIntTypedArrayConstructors(TA => {
  var BPE = TA.BYTES_PER_ELEMENT;
  var ab = new ArrayBuffer(BPE * 4, {maxByteLength: BPE * 5});
  var array = new TA(ab, BPE, 2);

  try {
    ab.resize(BPE * 5);
  } catch (_) {}

  // no error following grow:
  array.map(() => 0n);

  try {
    ab.resize(BPE * 3);
  } catch (_) {}

  // no error following shrink (within bounds):
  array.map(() => 0n);

  var expectedError;
  try {
    ab.resize(BPE * 3 - 1);
    // If the preceding "resize" operation is successful, the typed array will
    // be out out of bounds, so the subsequent prototype method should produce
    // a TypeError due to the semantics of ValidateTypedArray.
    expectedError = TypeError;
  } catch (_) {
    // The host is permitted to fail any "resize" operation at its own
    // discretion. If that occurs, the map operation should complete
    // successfully.
    expectedError = Test262Error;
  }

  assert.throws(expectedError, () => {
    array.map(() => 0n);
    throw new Test262Error('map completed successfully');
  });
}, null, ["passthrough"]);
