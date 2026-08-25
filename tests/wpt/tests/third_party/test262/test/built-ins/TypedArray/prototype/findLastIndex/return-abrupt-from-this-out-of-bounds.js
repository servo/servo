// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlastindex
description: Return abrupt when "this" value fails buffer boundary checks
includes: [testTypedArray.js]
features: [TypedArray, resizable-arraybuffer, array-find-from-last]
---*/

assert.sameValue(
  typeof TypedArray.prototype.findLastIndex,
  'function',
  'implements TypedArray.prototype.findLastIndex'
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
  array.findLastIndex(() => {});

  try {
    ab.resize(BPE * 3);
  } catch (_) {}

  // no error following shrink (within bounds):
  array.findLastIndex(() => {});

  var expectedError;
  try {
    ab.resize(BPE * 3 - 1);
    // If the preceding "resize" operation is successful, the typed array will
    // be out out of bounds, so the subsequent prototype method should produce
    // a TypeError due to the semantics of ValidateTypedArray.
    expectedError = TypeError;
  } catch (_) {
    // The host is permitted to fail any "resize" operation at its own
    // discretion. If that occurs, the findLastIndex operation should complete
    // successfully.
    expectedError = Test262Error;
  }

  assert.throws(expectedError, () => {
    array.findLastIndex(() => {});
    throw new Test262Error('findLastIndex completed successfully');
  });
}, null, ["passthrough"]);
