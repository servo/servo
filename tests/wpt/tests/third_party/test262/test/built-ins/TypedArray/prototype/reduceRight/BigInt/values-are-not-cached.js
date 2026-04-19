// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduceright
description: >
  Integer indexed values are not cached before iteration
info: |
  22.2.3.21 %TypedArray%.prototype.reduceRight ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduceRight is a distinct function that implements the
  same algorithm as Array.prototype.reduceRight as defined in 22.1.3.20 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.20 Array.prototype.reduceRight ( callbackfn [ , initialValue ] )

  ...
  8. Repeat, while k ≥ 0
    ...
    c. If kPresent is true, then
      i. Let kValue be ? Get(O, Pk).
      ii. Let accumulator be ? Call(callbackfn, undefined, « accumulator,
      kValue, k, O »).
    d. Decrease k by 1.
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([44n, 43n, 42n]));

  sample.reduceRight(function(a, v, i) {
    if (i > 0) {
      sample[i-1] = 42n;
    }

    assert.sameValue(
      v, 42n, "method does not cache values before callbackfn calls"
    );
  }, 0);
});
