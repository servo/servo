// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  Negative index is relative to the original typed array length.
info: |
  %TypedArray%.prototype.with ( index, value )

  ...
  4. Let relativeIndex be ? ToIntegerOrInfinity(index).
  5. If relativeIndex ≥ 0, let actualIndex be relativeIndex.
  6. Else, let actualIndex be len + relativeIndex.
  ...
  9. If IsValidIntegerIndex(O, 𝔽(actualIndex)) is false, throw a RangeError exception.
  ...
features: [TypedArray, change-array-by-copy, resizable-arraybuffer]
includes: [testTypedArray.js]
---*/

testWithTypedArrayConstructors(function(TA) {
  var byteLength = 4 * TA.BYTES_PER_ELEMENT;
  var rab = new ArrayBuffer(0, {maxByteLength: byteLength});
  var ta = new TA(rab);

  var value = {
    valueOf() {
      rab.resize(byteLength);
      return 123;
    }
  };

  assert.throws(RangeError, function() {
    ta.with(-1, value);
  });
}, null, ["passthrough"]);
