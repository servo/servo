// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: Error when target TypedArray fails boundary checks
includes: [testTypedArray.js]
features: [BigInt, TypedArray, resizable-arraybuffer]
---*/

assert.sameValue(
  typeof TypedArray.prototype.set,
  'function',
  'implements TypedArray.prototype.set'
);

assert.sameValue(
  typeof ArrayBuffer.prototype.resize,
  'function',
  'implements ArrayBuffer.prototype.resize'
);

testWithBigIntTypedArrayConstructors(TA => {
  var BPE = TA.BYTES_PER_ELEMENT;
  var ab = new ArrayBuffer(BPE * 4, {maxByteLength: BPE * 4});
  var target = new TA(ab, 0, 4);
  var source = new TA(new ArrayBuffer(BPE * 4));

  var expectedError;
  try {
    ab.resize(BPE * 3);
    expectedError = TypeError;
  } catch (_) {
    // The host is permitted to fail any "resize" operation at its own
    // discretion. If that occurs, the reverse operation should complete
    // successfully.
    expectedError = Test262Error;
  }

  assert.throws(expectedError, () => {
    target.set(source, 0);
    throw new Test262Error('The `set` operation completed successfully.');
  });
}, null, ["passthrough"]);
