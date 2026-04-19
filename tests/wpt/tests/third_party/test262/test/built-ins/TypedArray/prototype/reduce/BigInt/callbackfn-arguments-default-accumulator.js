// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduce
description: >
  callbackfn arguments using default accumulator (value at index 0)
info: |
  22.2.3.20 %TypedArray%.prototype.reduce ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduce is a distinct function that implements the same
  algorithm as Array.prototype.reduce as defined in 22.1.3.19 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.19 Array.prototype.reduce ( callbackfn [ , initialValue ] )

  ...
  7. Else initialValue is not present,
    a. Let kPresent be false.
    b. Repeat, while kPresent is false and k < len
      ...
      iii. If kPresent is true, then
        1. Let accumulator be ? Get(O, Pk).
      ...
  8. Repeat, while k < len
    ...
    c. If kPresent is true, then
      ...
      i. Let accumulator be ? Call(callbackfn, undefined, « accumulator, kValue,
      k, O »).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n, 44n]));

  var results = [];

  sample.reduce(function(accumulator) {
    results.push(arguments);
    return accumulator - 1n;
  });

  assert.sameValue(results.length, 2, "results.length");

  assert.sameValue(results[0].length, 4, "results[1].length");
  assert.sameValue(results[0][0], 42n, "results[1][0] - accumulator");
  assert.sameValue(results[0][1], 43n, "results[1][1] - kValue");
  assert.sameValue(results[0][2], 1, "results[1][2] - k");
  assert.sameValue(results[0][3], sample, "results[1][3] - this");

  assert.sameValue(results[1].length, 4, "results[2].length");
  assert.sameValue(results[1][0], 41n, "results[2][0] - accumulator");
  assert.sameValue(results[1][1], 44n, "results[2][1] - kValue");
  assert.sameValue(results[1][2], 2, "results[2][2] - k");
  assert.sameValue(results[1][3], sample, "results[2][3] - this");
});
