// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduce
description: >
  Returns last accumulator value
info: |
  22.2.3.20 %TypedArray%.prototype.reduce ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduce is a distinct function that implements the same
  algorithm as Array.prototype.reduce as defined in 22.1.3.19 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.19 Array.prototype.reduce ( callbackfn [ , initialValue ] )

  ...
  7. Else initialValue is not present,
    ...
    b. Repeat, while kPresent is false and k < len
      ...
      iii. If kPresent is true, then
        1. Let accumulator be ? Get(O, Pk).
      iv. Increase k by 1.
    ...
  8. Repeat, while k < len
    ...
    c. If kPresent is true, then
      i. Let kValue be ? Get(O, Pk).
      ii. Let accumulator be ? Call(callbackfn, undefined, « accumulator,
      kValue, k, O »).
  9. Return accumulator.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var calls, result;

  calls = 0;
  result = new TA(makeCtorArg([1, 2, 3])).reduce(function() {
    calls++;

    if (calls == 2) {
      return 42;
    }
  });
  assert.sameValue(result, 42, "using default accumulator");

  calls = 0;
  result = new TA(makeCtorArg([1, 2, 3])).reduce(function() {
    calls++;

    if (calls == 3) {
      return 7;
    }
  }, 0);
  assert.sameValue(result, 7, "using custom accumulator");
});
