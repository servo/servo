// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Returns true, even if index is out of bounds
info: |
  9.4.5.5 [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Perform ? IntegerIndexedElementSet(O, numericIndex, V).
      ii. Return true.
  ...

  9.4.5.11 IntegerIndexedElementSet ( O, index, value )

  ...
  8. Let length be the value of O's [[ArrayLength]] internal slot.
  9. If index < 0 or index ≥ length, return false.
  ...
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, Reflect, TypedArray]
---*/
testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n]));
  assert.sameValue(Reflect.set(sample, '-1', 1n), true, 'Reflect.set("new TA(makeCtorArg([42n]))", "-1", 1n) must return false');
  assert.sameValue(Reflect.set(sample, '1', 1n), true, 'Reflect.set("new TA(makeCtorArg([42n]))", "1", 1n) must return false');
  assert.sameValue(Reflect.set(sample, '2', 1n), true, 'Reflect.set("new TA(makeCtorArg([42n]))", "2", 1n) must return false');
  assert.sameValue(sample.hasOwnProperty('-1'), false, 'sample.hasOwnProperty("-1") must return false');
  assert.sameValue(sample.hasOwnProperty('1'), false, 'sample.hasOwnProperty("1") must return false');
  assert.sameValue(sample.hasOwnProperty('2'), false, 'sample.hasOwnProperty("2") must return false');
}, null, null, ["immutable"]);
