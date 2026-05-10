// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  %TypedArray%.prototype.with casts the index to an integer.
info: |
  %TypedArray%.prototype.with ( index, value )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  3. Let relativeIndex be ? ToIntegerOrInfinity(index).
  4. If index >= 0, let actualIndex be relativeIndex.
  5. Else, let actualIndex be len + relativeIndex.
  ...
features: [TypedArray, change-array-by-copy]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var arr = new TA(makeCtorArg([0, 4, 16]));

  assert.compareArray(arr.with(1.2, 7), [0, 7, 16]);
  assert.compareArray(arr.with("1", 3), [0, 3, 16]);
  assert.compareArray(arr.with("-1", 5), [0, 4, 5]);
  assert.compareArray(arr.with(NaN, 2), [2, 4, 16]);
  assert.compareArray(arr.with("dog", 33), [33, 4, 16]);
});
