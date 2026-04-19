// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-getownproperty-p
description: Returns undefined when P is not a valid index number.
info: |
  9.4.5.1 [[GetOwnProperty]] ( P )

  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. Let value be ? IntegerIndexedElementGet(O, numericIndex).
      ii. If value is undefined, return undefined.
  ...

  9.4.5.8 IntegerIndexedElementGet ( O, index )

  ...
  7. Let length be the value of O's [[ArrayLength]] internal slot.
  8. If index < 0 or index ≥ length, return undefined.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42]));

  assert.sameValue(Object.getOwnPropertyDescriptor(sample, "-1"), undefined);
  assert.sameValue(Object.getOwnPropertyDescriptor(sample, "-42"), undefined);
  assert.sameValue(Object.getOwnPropertyDescriptor(sample, "1"), undefined);
  assert.sameValue(Object.getOwnPropertyDescriptor(sample, "42"), undefined);
});
