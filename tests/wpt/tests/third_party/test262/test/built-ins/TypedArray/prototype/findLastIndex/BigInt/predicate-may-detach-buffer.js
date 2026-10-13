// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlastindex
description: >
  Predicate may detach the buffer
info: |
  %TypedArray%.prototype.findLastIndex ( predicate [ , thisArg ] )

  ...
  6. Repeat, while k ≥ 0,
    a. Let Pk be ! ToString(𝔽(k)).
    b. Let kValue be ! Get(O, Pk).
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
  ...

  IntegerIndexedElementGet ( O, index )

    Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
    If IsDetachedBuffer(buffer) is true, return undefined.

includes: [testTypedArray.js, detachArrayBuffer.js]
features: [BigInt, TypedArray, array-find-from-last]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));
  var loops = 0;

  sample.findLastIndex(function() {
    if (loops === 0) {
      $DETACHBUFFER(sample.buffer);
    }
    loops++;
  });
  assert.sameValue(loops, 2, "predicate is called once");
}, null, null, ["immutable"]);
