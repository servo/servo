// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.find
description: >
  Predicate may detach the buffer
info: |
  %TypedArray%.prototype.find (predicate [ , thisArg ] )

  %TypedArray%.prototype.find is a distinct function that implements the same
  algorithm as Array.prototype.find as defined in 22.1.3.8

  ...

  However, such optimization must not introduce any observable changes in the
  specified behaviour of the algorithm and must take into account the
  possibility that calls to predicate may cause the this value to become
  detached.


  Array.prototype.find ( predicate[ , thisArg ] )

    Let O be ? ToObject(this value).
    Let len be ? LengthOfArrayLike(O).
    If IsCallable(predicate) is false, throw a TypeError exception.
    Let k be 0.
    Repeat, while k < len,
      Let Pk be ! ToString(𝔽(k)).
      Let kValue be ? Get(O, Pk).
      Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
      If testResult is true, return kValue.
      Set k to k + 1.
    Return undefined.

  IntegerIndexedElementGet ( O, index )

    ...
    Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
    If IsDetachedBuffer(buffer) is true, return undefined.

includes: [testTypedArray.js, detachArrayBuffer.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var loops = 0;
  var sample = new TA(makeCtorArg(2));

  sample.find(function() {
    if (loops === 0) {
      $DETACHBUFFER(sample.buffer);
    }
    loops++;
  });

  assert.sameValue(loops, 2);
}, null, null, ["immutable"]);
