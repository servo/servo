// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduce
description: >
  Integer indexed values are not cached before iteration
info: |
  22.2.3.20 %TypedArray%.prototype.reduce ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduce is a distinct function that implements the same
  algorithm as Array.prototype.reduce as defined in 22.1.3.19 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.19 Array.prototype.reduce ( callbackfn [ , initialValue ] )

  ...
  8. Repeat, while k < len
    ...
    c. If kPresent is true, then
      i. Let kValue be ? Get(O, Pk).
      ii. Let accumulator be ? Call(callbackfn, undefined, « accumulator, kValue,
      k, O »).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n, 44n]));

  sample.reduce(function(a, v, i) {
    if (i < sample.length - 1) {
      sample[i+1] = 42n;
    }

    assert.sameValue(
      v, 42n, "method does not cache values before callbackfn calls"
    );
  }, 0);
});
