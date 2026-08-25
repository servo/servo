// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.every
description: >
  Returns false if any callbackfn call returns a coerced false.
info: |
  22.2.3.7 %TypedArray%.prototype.every ( callbackfn [ , thisArg ] )

  %TypedArray%.prototype.every is a distinct function that implements the same
  algorithm as Array.prototype.every as defined in 22.1.3.5 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length".

  22.1.3.5 Array.prototype.every ( callbackfn [ , thisArg ] )

  ...
  7. Return true.
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
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
    var result = sample.every(function() {
      called++;
      if (called === 1) {
        return true;
      }
      return val;
    });
    assert.sameValue(called, 2, "callbackfn called until it returned " + val);
    assert.sameValue(result, false, "result is false when it returned " + val);
  });
});
