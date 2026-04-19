// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-hasproperty-p
description: Return false if this has a detached buffer
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
features: [align-detached-buffer-semantics-with-web-reality, Reflect, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var sample = new TA(1);
  $DETACHBUFFER(sample.buffer);

  assert.sameValue(Reflect.has(sample, "0"), false, 'Reflect.has(sample, "0") must return false');
  assert.sameValue(Reflect.has(sample, "-0"), false, 'Reflect.has(sample, "-0") must return false');
  assert.sameValue(Reflect.has(sample, "1.1"), false, 'Reflect.has(sample, "1.1") must return false');
}, null, ["passthrough"]);
