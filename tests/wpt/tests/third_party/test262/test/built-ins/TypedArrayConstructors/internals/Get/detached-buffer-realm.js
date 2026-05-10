// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-get-p-receiver
description: >
  Returns undefined if key has a numeric index and object has a detached
  buffer (honoring the Realm of the current execution context)
info: |
  [[Get]] ( P, Receiver )

    If Type(P) is String, then
      Let numericIndex be ! CanonicalNumericIndexString(P).
      If numericIndex is not undefined, then
        Return ! IntegerIndexedElementGet(O, numericIndex).

  IntegerIndexedElementGet ( O, index )

    Assert: O is an Integer-Indexed exotic object.
    Assert: Type(index) is Number.
    Let buffer be O.[[ViewedArrayBuffer]].
    If IsDetachedBuffer(buffer) is true, return undefined.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, cross-realm, TypedArray]
---*/

let other = $262.createRealm().global;

testWithTypedArrayConstructors(function(TA) {
  let OtherTA = other[TA.name];
  let sample = new OtherTA(1);

  $DETACHBUFFER(sample.buffer);

  assert.sameValue(sample[0], undefined, 'The value of sample[0] is expected to equal `undefined`');
}, null, ["passthrough"]);
