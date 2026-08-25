// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduceright
description: >
  callbackfn arguments using default accumulator (value at last index)
info: |
  22.2.3.21 %TypedArray%.prototype.reduceRight ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduceRight is a distinct function that implements the
  same algorithm as Array.prototype.reduceRight as defined in 22.1.3.20 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.20 Array.prototype.reduceRight ( callbackfn [ , initialValue ] )

  ...
  7. Else initialValue is not present,
    ...
    b. Repeat, while kPresent is false and k ≥ 0
      ...
      ii. Let kPresent be ? HasProperty(O, Pk).
      iii. If kPresent is true, then
        1. Let accumulator be ? Get(O, Pk).
      iv. Decrease k by 1.
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
  var sample = new TA(makeCtorArg([42n, 43n, 44n]));

  var results = [];

  sample.reduceRight(function(accumulator) {
    results.push(arguments);
    return accumulator + 1n;
  });

  assert.sameValue(results.length, 2, "results.length");

  assert.sameValue(results[0].length, 4, "results[1].length");
  assert.sameValue(results[0][0], 44n, "results[1][0] - accumulator");
  assert.sameValue(results[0][1], 43n, "results[1][1] - kValue");
  assert.sameValue(results[0][2], 1, "results[1][2] - k");
  assert.sameValue(results[0][3], sample, "results[1][3] - this");

  assert.sameValue(results[1].length, 4, "results[2].length");
  assert.sameValue(results[1][0], 45n, "results[2][0] - accumulator");
  assert.sameValue(results[1][1], 42n, "results[2][1] - kValue");
  assert.sameValue(results[1][2], 0, "results[2][2] - k");
  assert.sameValue(results[1][3], sample, "results[2][3] - this");
});
