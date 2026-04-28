// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-integer-indexed-exotic-objects-getownproperty-p
description: Test for-in enumeration with detached buffer.
info: |
  9.4.5.1 [[GetOwnProperty]] ( P )
    ...
    3. If Type(P) is String, then
      a. Let numericIndex be ! CanonicalNumericIndexString(P).
      b. If numericIndex is not undefined, then
        i. Let value be ! IntegerIndexedElementGet(O, numericIndex).
    ...

  IntegerIndexedElementGet ( O, index )
    ...
    Let buffer be O.[[ViewedArrayBuffer]].
    If IsDetachedBuffer(buffer) is true, return undefined.
    ...

  13.7.5.15 EnumerateObjectProperties (O)
    ...
    EnumerateObjectProperties must obtain the own property keys of the
    target object by calling its [[OwnPropertyKeys]] internal method.
    Property attributes of the target object must be obtained by
    calling its [[GetOwnProperty]] internal method.

includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var sample = new TA(42);
  $DETACHBUFFER(sample.buffer);

  let counter = 0;
  for (var key in sample) {
    counter++;
  }
  assert.sameValue(counter, 0, 'The value of `counter` is 0');
}, null, ["passthrough"]);
