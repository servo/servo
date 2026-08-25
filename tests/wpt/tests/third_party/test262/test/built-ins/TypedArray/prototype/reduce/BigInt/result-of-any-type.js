// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduce
description: >
  Result can be of any type without any number conversions
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
features: [BigInt, Symbol, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n, 44n]));
  [
    ["test262", "string"],
    ["", "empty string"],
    [undefined, "undefined"],
    [null, "null"],
    [-0, "-0"],
    [42, "integer"],
    [NaN, "NaN"],
    [Infinity, "Infinity"],
    [0.6, "float number"],
    [true, "true"],
    [false, "false"],
    [Symbol(""), "symbol"],
    [{}, "object"]
  ].forEach(function(item) {
    var result;

    result = sample.reduce(function() {
      return item[0];
    });
    assert.sameValue(result, item[0], item[1] + " - using default accumulator");

    result = sample.reduce(function() {
      return item[0];
    }, 0);

    assert.sameValue(result, item[0], item[1] + " - using custom accumulator");
  });
});
