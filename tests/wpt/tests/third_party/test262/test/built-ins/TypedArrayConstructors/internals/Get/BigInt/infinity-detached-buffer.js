// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-integerindexedelementget
description: >
  "Infinity" is a canonical numeric string, test with access on detached buffer.
info: |
  9.4.5.4 [[Get]] ( P, Receiver )
  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Return ? IntegerIndexedElementGet(O, numericIndex).
  ...

  7.1.16 CanonicalNumericIndexString ( argument )
    ...
    3. Let n be ! ToNumber(argument).
    4. If SameValue(! ToString(n), argument) is false, return undefined.
    5. Return n.

  9.4.5.8 IntegerIndexedElementGet ( O, index )
    ...
    3. Let buffer be O.[[ViewedArrayBuffer]].
    4. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
    ...

includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  let sample = new TA(0);
  $DETACHBUFFER(sample.buffer);

  assert.sameValue(sample.Infinity, undefined, 'The value of sample.Infinity is expected to equal `undefined`');
}, null, ["passthrough"]);
