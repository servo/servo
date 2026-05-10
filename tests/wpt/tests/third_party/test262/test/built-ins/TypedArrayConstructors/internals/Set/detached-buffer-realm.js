// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Throws a TypeError if key has a numeric index and object has a detached
  buffer (honoring the Realm of the current execution context)
info: |
  9.4.5.5 [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Perform ? IntegerIndexedElementSet(O, numericIndex, V).
      ii. Return true.
  ...

  IntegerIndexedElementSet ( O, index, value )

  Assert: O is an Integer-Indexed exotic object.
  Assert: Type(index) is Number.
  If O.[[ContentType]] is BigInt, let numValue be ? ToBigInt(value).
  Otherwise, let numValue be ? ToNumber(value).
  Let buffer be O.[[ViewedArrayBuffer]].
  If IsDetachedBuffer(buffer) is true, return false.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, cross-realm, TypedArray]
---*/

let other = $262.createRealm().global;
testWithTypedArrayConstructors(function(TA) {
  let OtherTA = other[TA.name];
  let sample = new OtherTA(1);
  $DETACHBUFFER(sample.buffer);
  sample[0] = 1;
  assert.sameValue(sample[0], undefined, '`sample[0]` is undefined');
}, null, ["passthrough"]);
