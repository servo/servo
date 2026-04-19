// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduceright
description: >
  Result can be of any type without any number conversions
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
  9. Return accumulator.
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 43, 44]));
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

    result = sample.reduceRight(function() {
      return item[0];
    });
    assert.sameValue(result, item[0], item[1] + " - using default accumulator");

    result = sample.reduceRight(function() {
      return item[0];
    }, 0);

    assert.sameValue(result, item[0], item[1] + " - using custom accumulator");
  });
});
