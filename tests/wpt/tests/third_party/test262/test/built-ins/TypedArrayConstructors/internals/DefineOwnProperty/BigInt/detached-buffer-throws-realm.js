// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Throws TypeError for valid descriptor & canonical numeric string if buffer is detached.
  (honoring the Realm of the current execution context)
info: |
  [[DefineOwnProperty]] ( P, Desc )

  [...]
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. If ! IsValidIntegerIndex(O, numericIndex) is false, return false.

  IsValidIntegerIndex ( O, index )

  [...]
  2. If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is true, return false.
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray]
---*/

var other = $262.createRealm().global;

testWithBigIntTypedArrayConstructors(function(TA) {
  var OtherTA = other[TA.name];
  var sample = new OtherTA([0n]);
  var desc = Object.getOwnPropertyDescriptor(sample, "0");
  $DETACHBUFFER(sample.buffer);

  assert.throws(TypeError, function() {
    Object.defineProperty(sample, "0", desc);
  });

  assert.throws(TypeError, function() {
    Object.defineProperty(sample, "1", desc);
  });

  assert.throws(TypeError, function() {
    Object.defineProperty(sample, "-1", desc);
  });

  assert.throws(TypeError, function() {
    Object.defineProperty(sample, "1.5", desc);
  });

  assert.throws(TypeError, function() {
    Object.defineProperty(sample, "-0", desc);
  });
}, null, ["passthrough"]);
