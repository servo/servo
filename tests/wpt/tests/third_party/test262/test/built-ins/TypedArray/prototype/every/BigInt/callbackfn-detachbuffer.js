// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.every
description: >
  Instance buffer can be detached during loop
info: |
  22.2.3.7 %TypedArray%.prototype.every ( callbackfn [ , thisArg ] )

  %TypedArray%.prototype.every is a distinct function that implements the same
  algorithm as Array.prototype.every as defined in 22.1.3.5 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length".

  22.1.3.5 Array.prototype.every ( callbackfn [ , thisArg ] )

  ...
  6. Repeat, while k < len
    ...
    c. If kPresent is true, then
      i. Let kValue be ? Get(O, Pk).
      ii. Let testResult be ToBoolean(? Call(callbackfn, T, « kValue, k, O »)).
  ...
includes: [detachArrayBuffer.js, testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var loops = 0;
  var sample = new TA(2);

  sample.every(function() {
    if (loops === 0) {
      $DETACHBUFFER(sample.buffer);
    }
    loops++;
    return true;
  });

  assert.sameValue(loops, 2);
}, null, ["passthrough"]);
