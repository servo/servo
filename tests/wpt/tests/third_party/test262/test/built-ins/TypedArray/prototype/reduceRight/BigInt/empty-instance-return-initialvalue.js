// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduceright
description: >
  Returns given initialValue on empty instances without calling callbackfn
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
  9. Return accumulator.
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var called = false;
  var result = new TA(makeCtorArg(0)).reduceRight(function() {
    called = true;
  }, 42);

  assert.sameValue(result, 42);
  assert.sameValue(called, false);
});
