// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  %TypedArray%.prototype.with throws if the (negative) index is smaller than -length.
info: |
  %TypedArray%.prototype.with ( index, value )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  3. Let relativeIndex be ? ToIntegerOrInfinity(index).
  4. If index >= 0, let actualIndex be relativeIndex.
  5. Else, let actualIndex be len + relativeIndex.
  6. If actualIndex >= len or actualIndex < 0, throw a *RangeError* exception.
  ...
features: [TypedArray, change-array-by-copy]
includes: [testTypedArray.js]
---*/

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var arr = new TA(makeCtorArg([0, 1, 2]));

  assert.throws(RangeError, function() {
    arr.with(-4, 7);
  });

  assert.throws(RangeError, function() {
    arr.with(-10, 7);
  });

  assert.throws(RangeError, function() {
    arr.with(-(2 ** 53) - 2, 7);
  });

  assert.throws(RangeError, function() {
    arr.with(-Infinity, 7);
  });
});
