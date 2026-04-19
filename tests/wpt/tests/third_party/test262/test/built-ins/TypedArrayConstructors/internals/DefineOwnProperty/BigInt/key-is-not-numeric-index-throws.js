// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Throws TypeError for valid descriptor & canonical numeric string that is invalid index.
info: |
  [[DefineOwnProperty]] ( P, Desc )

  [...]
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. If ! IsValidIntegerIndex(O, numericIndex) is false, return false.

  IsValidIntegerIndex ( O, index )

  [...]
  3. If ! IsIntegralNumber(index) is false, return false.
  4. If index is -0𝔽, return false.
  5. If ℝ(index) < 0 or ℝ(index) ≥ O.[[ArrayLength]], return false.
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([0n]));
  var desc = Object.getOwnPropertyDescriptor(sample, "0");

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
