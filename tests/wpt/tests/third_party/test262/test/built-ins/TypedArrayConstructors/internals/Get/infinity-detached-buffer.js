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

  CanonicalNumericIndexString ( argument )
    ...
    3. Let n be ! ToNumber(argument).
    4. If SameValue(! ToString(n), argument) is false, return undefined.
    5. Return n.

  IntegerIndexedElementGet ( O, index )

    Assert: O is an Integer-Indexed exotic object.
    Assert: Type(index) is Number.
    Let buffer be O.[[ViewedArrayBuffer]].
    If IsDetachedBuffer(buffer) is true, return undefined.

includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  let sample = new TA(0);
  $DETACHBUFFER(sample.buffer);

  assert.sameValue(sample.Infinity, undefined, 'The value of sample.Infinity is expected to equal `undefined`');
}, null, ["passthrough"]);
