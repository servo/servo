// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Returns true, even if index is -0
info: |
  [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Perform ? IntegerIndexedElementSet(O, numericIndex, V).
      ii. Return true.
  ...
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, Reflect, TypedArray]
---*/
testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n]));
  assert.sameValue(Reflect.set(sample, '-0', 1n), true, 'Reflect.set("new TA(makeCtorArg([42n]))", "-0", 1n) must return true');
  assert.sameValue(sample.hasOwnProperty('-0'), false, 'sample.hasOwnProperty("-0") must return false');
}, null, null, ["immutable"]);
