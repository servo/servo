// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  IsValidIntegerIndex is checked after all coercions happened.
info: |
  %TypedArray%.prototype.with ( index, value )

  ...
  8. Else, let numericValue be ? ToNumber(value).
  9. If IsValidIntegerIndex(O, 𝔽(actualIndex)) is false, throw a RangeError exception.
  ...
features: [TypedArray, change-array-by-copy, resizable-arraybuffer]
includes: [testTypedArray.js]
---*/

testWithTypedArrayConstructors(TA => {
  var rab = new ArrayBuffer(0, {maxByteLength: TA.BYTES_PER_ELEMENT});
  var ta = new TA(rab);
  assert.sameValue(ta.length, 0);

  var value = {
    valueOf() {
      rab.resize(TA.BYTES_PER_ELEMENT);
      return 0;
    }
  };

  var result = ta.with(0, value);

  assert.sameValue(result.length, 0);
  assert.sameValue(rab.byteLength, TA.BYTES_PER_ELEMENT);
}, null, ["passthrough"]);
