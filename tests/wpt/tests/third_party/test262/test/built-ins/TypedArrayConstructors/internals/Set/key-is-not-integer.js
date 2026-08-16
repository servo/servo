// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Returns true, even if index is not CanonicalNumericIndexString
info: |
  9.4.5.5 [[Set]] ( P, V, Receiver)

  ...
  2. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Perform ? IntegerIndexedElementSet(O, numericIndex, V).
      ii. Return true.
  ...

includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, Reflect, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42]));

  assert.sameValue(Reflect.set(sample, "1.1", 1), true, 'Reflect.set(sample, "1.1", 1) must return true');
  assert.sameValue(Reflect.set(sample, "0.0001", 1), true, 'Reflect.set(sample, "0.0001", 1) must return true');

  assert.sameValue(sample.hasOwnProperty("1.1"), false, 'sample.hasOwnProperty("1.1") must return false');
  assert.sameValue(
    sample.hasOwnProperty("0.0001"),
    false,
    'sample.hasOwnProperty("0.0001") must return false'
  );
}, null, null, ["immutable"]);
