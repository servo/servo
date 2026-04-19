// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlast
description: >
  Predicate may detach the buffer
info: |
  %TypedArray%.prototype.findLast (predicate [ , thisArg ] )

  ...

  However, such optimization must not introduce any observable changes in the
  specified behaviour of the algorithm and must take into account the
  possibility that calls to predicate may cause the this value to become
  detached.

  IntegerIndexedElementGet ( O, index )

    ...
    Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
    If IsDetachedBuffer(buffer) is true, return undefined.

includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray, array-find-from-last]
---*/

testWithTypedArrayConstructors(function(TA) {
  var loops = 0;
  var sample = new TA(2);

  sample.findLast(function() {
    if (loops === 0) {
      $DETACHBUFFER(sample.buffer);
    }
    loops++;
  });

  assert.sameValue(loops, 2);
}, null, ["passthrough"]);
