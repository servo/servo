// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.foreach
description: >
  Instance buffer can be detached during loop
info: |
  22.2.3.12 %TypedArray%.prototype.forEach ( callbackfn [ , thisArg ] )

  %TypedArray%.prototype.forEach is a distinct function that implements the same
  algorithm as Array.prototype.forEach as defined in 22.1.3.10 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length"

  22.1.3.10 Array.prototype.forEach ( callbackfn [ , thisArg ] )

  ...
  6. Repeat, while k < len
    ...
    c. If kPresent is true, then
      ...
      ii. Perform ? Call(callbackfn, T, « kValue, k, O »).
  ...
includes: [detachArrayBuffer.js, testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var loops = 0;
  var sample = new TA(makeCtorArg(2));

  sample.forEach(function() {
    if (loops === 0) {
      $DETACHBUFFER(sample.buffer);
    }
    loops++;
  });

  assert.sameValue(loops, 2);
}, null, null, ["immutable"]);
