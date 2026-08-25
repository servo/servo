// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduceright
description: >
  If len is 0 and initialValue is not present, throw a TypeError exception.
info: |
  22.2.3.21 %TypedArray%.prototype.reduceRight ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduceRight is a distinct function that implements the
  same algorithm as Array.prototype.reduceRight as defined in 22.1.3.20 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.20 Array.prototype.reduceRight ( callbackfn [ , initialValue ] )

  ...
  4. If len is 0 and initialValue is not present, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var called = 0;

  var ta = new TA(makeCtorArg(0));
  assert.throws(TypeError, function() {
    ta.reduceRight(function() {
      called++;
    });
  });

  assert.sameValue(called, 0);
});
