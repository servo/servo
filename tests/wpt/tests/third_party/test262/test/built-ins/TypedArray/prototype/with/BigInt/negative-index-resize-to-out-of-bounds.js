// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  A negative index is relative to the original length and throws a RangeError
  if value coercion shrinks the buffer so that the index is out of bounds.
features: [BigInt, TypedArray, change-array-by-copy, resizable-arraybuffer]
includes: [testTypedArray.js]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var byteLength = 2 * TA.BYTES_PER_ELEMENT;
  var buffer = new ArrayBuffer(byteLength, {maxByteLength: byteLength});
  var sample = new TA(buffer);

  var value = {
    valueOf: function() {
      buffer.resize(TA.BYTES_PER_ELEMENT);
      return 0n;
    }
  };

  // -1 resolves to index 1 before shrinking. Reject that index before copying
  // missing elements, which would instead throw a TypeError for BigInt arrays.
  assert.throws(RangeError, function() {
    sample.with(-1, value);
  });
}, null, ["passthrough"]);
