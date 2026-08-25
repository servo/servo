// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Throws TypeError for valid descriptor & canonical numeric string if buffer is detached.
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
features: [align-detached-buffer-semantics-with-web-reality, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var sample = new TA([0]);
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
