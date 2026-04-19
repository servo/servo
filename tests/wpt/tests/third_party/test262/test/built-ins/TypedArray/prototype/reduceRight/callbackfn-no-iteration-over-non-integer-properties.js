// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduceright
description: >
  Does not iterate over non-integer properties
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
features: [Symbol, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([7, 8]));

  var results = [];

  sample.foo = 42;
  sample[Symbol("1")] = 43;

  sample.reduceRight(function() {
    results.push(arguments);
  }, 0);

  assert.sameValue(results.length, 2, "results.length");

  assert.sameValue(results[0][2], 1, "results[0][2] - k");
  assert.sameValue(results[1][2], 0, "results[1][2] - k");

  assert.sameValue(results[0][1], 8, "results[0][1] - kValue");
  assert.sameValue(results[1][1], 7, "results[1][1] - kValue");
});
