// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  Copying undefined throws a TypeError after index coercion shrinks the buffer.
includes: [testTypedArray.js]
features: [BigInt, TypedArray, change-array-by-copy, resizable-arraybuffer]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var buffer = new ArrayBuffer(3 * TA.BYTES_PER_ELEMENT, {
    maxByteLength: 3 * TA.BYTES_PER_ELEMENT
  });
  var sample = new TA(buffer);
  sample[0] = 1n;
  sample[1] = 2n;
  sample[2] = 3n;
  var calls = 0;
  var index = {
    valueOf: function() {
      calls++;
      buffer.resize(TA.BYTES_PER_ELEMENT);
      return 0;
    }
  };

  // Index 0 remains valid, but copying the remaining elements reads undefined.
  assert.throws(TypeError, function() {
    sample.with(index, 9n);
  });
  assert.sameValue(calls, 1, "index is coerced once");
  assert.compareArray(sample, [1n]);
}, null, ["passthrough"]);
