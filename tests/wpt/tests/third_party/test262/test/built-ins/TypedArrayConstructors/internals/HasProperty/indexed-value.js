// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-hasproperty-p
description: >
  Return true for indexed properties
info: |
  9.4.5.2 [[HasProperty]](P)

  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Let buffer be O.[[ViewedArrayBuffer]].
      ii. If IsDetachedBuffer(buffer) is true, return false.
      iii. If ! IsValidIntegerIndex(O, numericIndex) is false, return false.
      iv. Return true.
  ...
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, Reflect, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 43]));

  assert.sameValue(Reflect.has(sample, 0), true, 'Reflect.has("new TA(makeCtorArg([42, 43]))", 0) must return true');
  assert.sameValue(Reflect.has(sample, 1), true, 'Reflect.has("new TA(makeCtorArg([42, 43]))", 1) must return true');
});
