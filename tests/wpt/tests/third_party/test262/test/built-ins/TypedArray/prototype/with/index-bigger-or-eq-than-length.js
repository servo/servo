// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typed%array%.prototype.with
description: >
  %TypedArray%.prototype.with throws if the index is bigger than or equal to the array length.
info: |
  %TypedArray%.prototype.with ( index, value )

  ...
  3. Let len be O.[[ArrayLength]].
  3. Let relativeIndex be ? ToIntegerOrInfinity(index).
  4. If index >= 0, let actualIndex be relativeIndex.
  5. Else, let actualIndex be len + relativeIndex.
  6. If ! IsValidIntegerIndex(O, actualIndex) is false, throw a *RangeError* exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray, change-array-by-copy]
---*/

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var arr = new TA(makeCtorArg([0, 1, 2]));

  assert.throws(RangeError, function() {
    arr.with(3, 7);
  });

  assert.throws(RangeError, function() {
    arr.with(10, 7);
  });

  assert.throws(RangeError, function() {
    arr.with(2 ** 53 + 2, 7);
  });

  assert.throws(RangeError, function() {
    arr.with(Infinity, 7);
  });
});
