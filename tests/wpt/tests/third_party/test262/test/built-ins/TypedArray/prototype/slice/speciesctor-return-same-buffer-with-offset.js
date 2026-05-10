// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.slice
description: >
  When species constructs a typed array using the same buffer but with a
  different byte offset, slice output reflects element-by-element copying into
  that buffer.
info: |
  %TypedArray%.prototype.slice ( start, end )

  ...
  14. If countBytes > 0, then
    g. If srcType is targetType, then
      ix. Repeat, while targetByteIndex < endByteIndex,
        1. Let value be GetValueFromBuffer(srcBuffer, srcByteIndex, uint8, true, unordered).
        2. Perform SetValueInBuffer(targetBuffer, targetByteIndex, uint8, value, true, unordered).
        ...
features: [TypedArray]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithTypedArrayConstructors(function(TA) {
  var ta = new TA([
    10,
    20,
    30,
    40,
    50,
    60,
  ]);

  ta.constructor = {
    [Symbol.species]: function() {
      return new TA(ta.buffer, 2 * TA.BYTES_PER_ELEMENT);
    }
  };

  var result = ta.slice(1, 4);

  assert.compareArray(result, [
    20, 20, 20, 60,
  ]);
}, null, ["passthrough"]);
