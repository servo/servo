// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.some
description: >
  Returns false if every callbackfn calls returns a coerced false.
info: |
  22.2.3.25 %TypedArray%.prototype.some ( callbackfn [ , thisArg ] )

  %TypedArray%.prototype.some is a distinct function that implements the same
  algorithm as Array.prototype.some as defined in 22.1.3.24 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length".

  22.1.3.24 Array.prototype.some ( callbackfn [ , thisArg ] )

  ...
  7. Return true.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(42));

  [
    false,
    "",
    0,
    -0,
    NaN,
    undefined,
    null
  ].forEach(function(val) {
    var called = 0;
    var result = sample.some(function() {
      called++;
      return val;
    });
    assert.sameValue(called, 42, "callbackfn called for each index property");
    assert.sameValue(result, false, "result is false - " + val);
  });
});
