// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.tolocalestring
description: Return abrupt when "this" value fails buffer boundary checks
includes: [testTypedArray.js]
features: [ArrayBuffer, TypedArray, arrow-function, resizable-arraybuffer]
---*/

assert.sameValue(
  typeof TypedArray.prototype.toLocaleString,
  'function',
  'implements TypedArray.prototype.toLocaleString'
);

assert.sameValue(
  typeof ArrayBuffer.prototype.resize,
  'function',
  'implements ArrayBuffer.prototype.resize'
);

testWithTypedArrayConstructors(TA => {
  var BPE = TA.BYTES_PER_ELEMENT;
  var ab = new ArrayBuffer(BPE * 4, {maxByteLength: BPE * 5});
  var array = new TA(ab, BPE, 2);

  try {
    ab.resize(BPE * 5);
  } catch (_) {}

  // no error following grow:
  array.toLocaleString();

  try {
    ab.resize(BPE * 3);
  } catch (_) {}

  // no error following shrink (within bounds):
  array.toLocaleString();

  var expectedError;
  try {
    ab.resize(BPE * 3 - 1);
    // If the preceding "resize" operation is successful, the typed array will
    // be out out of bounds, so the subsequent prototype method should produce
    // a TypeError due to the semantics of ValidateTypedArray.
    expectedError = TypeError;
  } catch (_) {
    // The host is permitted to fail any "resize" operation at its own
    // discretion. If that occurs, the toLocaleString operation should complete
    // successfully.
    expectedError = Test262Error;
  }

  assert.throws(expectedError, () => {
    array.toLocaleString();
    throw new Test262Error('toLocaleString completed successfully');
  });
}, null, ["passthrough"]);
