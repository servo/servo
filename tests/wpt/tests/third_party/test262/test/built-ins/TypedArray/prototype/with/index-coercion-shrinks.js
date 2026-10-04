// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  Missing elements are converted to NaN or zero after index coercion shrinks the buffer.
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray, change-array-by-copy, resizable-arraybuffer]
---*/

testWithTypedArrayConstructors(function(TA) {
  var buffer = new ArrayBuffer(3 * TA.BYTES_PER_ELEMENT, {
    maxByteLength: 3 * TA.BYTES_PER_ELEMENT
  });
  var sample = new TA(buffer);
  sample[0] = 1;
  sample[1] = 2;
  sample[2] = 3;
  var calls = 0;
  var index = {
    valueOf: function() {
      calls++;
      buffer.resize(TA.BYTES_PER_ELEMENT);
      return 0;
    }
  };

  // Index 0 remains valid, but copying the remaining elements reads undefined.
  var result = sample.with(index, 9);
  var converted = isFloatTypedArrayConstructor(TA) ? NaN : 0;
  assert.compareArray(result, [9, converted, converted]);
  assert.sameValue(calls, 1, "index is coerced once");
  assert.compareArray(sample, [1]);
}, null, ["passthrough"]);
