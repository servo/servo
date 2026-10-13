// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduceright
description: >
  Instance buffer can be detached during loop
info: |
  22.2.3.21 %TypedArray%.prototype.reduceRight ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduceRight is a distinct function that implements the
  same algorithm as Array.prototype.reduceRight as defined in 22.1.3.20 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.20 Array.prototype.reduceRight ( callbackfn [ , initialValue ] )

  ...
  8. Repeat, while k < len
    ...
    c. If kPresent is true, then
      ...
      i. Let accumulator be ? Call(callbackfn, undefined, « accumulator, kValue,
      k, O »).
  ...
includes: [detachArrayBuffer.js, testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var loops = 0;
  var sample = new TA(makeCtorArg(2));

  sample.reduceRight(function() {
    if (loops === 0) {
      $DETACHBUFFER(sample.buffer);
    }
    loops++;
    return true;
  }, 0);

  assert.sameValue(loops, 2);
}, null, null, ["immutable"]);
