// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-getownproperty-p
description: >
  Returned undefined if this has a detached buffer (honoring the Realm of the current execution context)
info: |
  9.4.5.1 [[GetOwnProperty]] ( P )

  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Let value be ! IntegerIndexedElementGet(O, numericIndex).
      ii. If value is undefined, return undefined.
  ...

  IntegerIndexedElementGet ( O, index )

  ...
  Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
  If IsDetachedBuffer(buffer) is true, return undefined.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, cross-realm, TypedArray]
---*/

var other = $262.createRealm().global;

testWithBigIntTypedArrayConstructors(function(TA) {
  var OtherTA = other[TA.name];
  var sample = new OtherTA(1);

  $DETACHBUFFER(sample.buffer);

  assert.sameValue(
    Object.getOwnPropertyDescriptor(sample, 0),
    undefined,
    'Object.getOwnPropertyDescriptor("new OtherTA(1)", 0) must return undefined'
  );
}, null, ["passthrough"]);
