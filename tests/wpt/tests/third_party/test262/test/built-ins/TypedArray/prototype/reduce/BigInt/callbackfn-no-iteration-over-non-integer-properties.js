// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduce
description: >
  Does not iterate over non-integer properties
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
      ...
      i. Let accumulator be ? Call(callbackfn, undefined, « accumulator, kValue,
      k, O »).
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([7n, 8n]));

  var results = [];

  sample.foo = 42;
  sample[Symbol("1")] = 43;

  sample.reduce(function() {
    results.push(arguments);
  }, 0);

  assert.sameValue(results.length, 2, "results.length");

  assert.sameValue(results[0][2], 0, "results[0][2] - k");
  assert.sameValue(results[1][2], 1, "results[1][2] - k");

  assert.sameValue(results[0][1], 7n, "results[0][1] - kValue");
  assert.sameValue(results[1][1], 8n, "results[1][1] - kValue");
});
