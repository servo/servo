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
features: [BigInt, TypedArray, change-array-by-copy]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var arr = new TA(makeCtorArg([0n, 1n, 2n]));

  var value = {
    valueOf() {
      arr[0] = 3n;
      return 4n;
    }
  };

  assert.compareArray(arr.with(1, value), [3n, 4n, 2n]);
  assert.compareArray(arr, [3n, 1n, 2n]);
}, null, null, ["immutable"]);
