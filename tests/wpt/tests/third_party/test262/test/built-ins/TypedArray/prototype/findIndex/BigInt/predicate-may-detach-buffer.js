// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2021 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findindex
description: >
  Predicate may detach the buffer
info: |
  22.2.3.11 %TypedArray%.prototype.findIndex ( predicate [ , thisArg ] )

  %TypedArray%.prototype.findIndex is a distinct function that implements the
  same algorithm as Array.prototype.findIndex as defined in 22.1.3.9 except that
  the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  ...

  22.1.3.9 Array.prototype.findIndex ( predicate[ , thisArg ] )

  ...
  6. Repeat, while k < len
    ...
    b. Let kValue be ? Get(O, Pk).
    c. Let testResult be ToBoolean(? Call(predicate, T, « kValue, k, O »)).
  ...

  9.4.5.8 IntegerIndexedElementGet ( O, index )

  ...
  3. Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
  4. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var sample = new TA(2);
  var loops = 0;

  sample.findIndex(function() {
    if (loops === 0) {
      $DETACHBUFFER(sample.buffer);
    }
    loops++;
  });
  assert.sameValue(loops, 2, "predicate is called once");
}, null, ["passthrough"]);
