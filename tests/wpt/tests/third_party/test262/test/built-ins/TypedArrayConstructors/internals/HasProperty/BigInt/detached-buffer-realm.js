// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-hasproperty-p
description: >
  Returns false if this has a detached buffer (honoring the Realm of the
  current execution context)
info: |
  9.4.5.2 [[HasProperty]](P)

  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Let buffer be O.[[ViewedArrayBuffer]].
      ii. If IsDetachedBuffer(buffer) is true, return false.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, cross-realm, Reflect, TypedArray]
---*/

var other = $262.createRealm().global;

testWithBigIntTypedArrayConstructors(function(TA) {
  var OtherTA = other[TA.name];
  var sample = new OtherTA(1);

  $DETACHBUFFER(sample.buffer);

  assert.sameValue(Reflect.has(sample, '0'), false, 'Reflect.has(sample, "0") must return false');
}, null, ["passthrough"]);
