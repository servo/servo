// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.some
description: >
  Returns true if any callbackfn returns a coerced true.
info: |
  22.2.3.25 %TypedArray%.prototype.some ( callbackfn [ , thisArg ] )

  %TypedArray%.prototype.some is a distinct function that implements the same
  algorithm as Array.prototype.some as defined in 22.1.3.24 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length".

  22.1.3.24 Array.prototype.some ( callbackfn [ , thisArg ] )

  ...
  6. Repeat, while k < len
    ...
    c. If kPresent is true, then
      i. Let kValue be ? Get(O, Pk).
      ii. Let testResult be ToBoolean(? Call(callbackfn, T, « kValue, k, O »)).
      iii. If testResult is true, return true.
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray]
---*/

var s = Symbol("1");

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(42));
  [
    true,
    1,
    "test262",
    s,
    {},
    [],
    -1,
    Infinity,
    -Infinity,
    0.1,
    -0.1
  ].forEach(function(val) {
    var called = 0;
    var result = sample.some(function() {
      called++;
      if (called == 1) {
        return false;
      }
      return val;
    });
    assert.sameValue(called, 2, "callbackfn called for each index property");

    var msg = "result is true - " + (val === s ? "symbol" : val);
    assert.sameValue(result, true, msg);
  });
});
