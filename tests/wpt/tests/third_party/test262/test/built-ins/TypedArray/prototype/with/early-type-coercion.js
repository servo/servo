// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  %TypedArray%.prototype.with invokes ToNumber before copying
info: |
  %TypedArray%.prototype.with ( index, value )

  ...
  7. If _O_.[[ContentType]] is ~BigInt~, set _value_ to ? ToBigInt(_value_).
  8. Else, set _value_ to ? ToNumber(_value_).
  ...
features: [TypedArray, change-array-by-copy]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var arr = new TA(makeCtorArg([0, 1, 2]));

  var value = {
    valueOf() {
      arr[0] = 3;
      return 4;
    }
  };

  assert.compareArray(arr.with(1, value), [3, 4, 2]);
  assert.compareArray(arr, [3, 1, 2]);
});
