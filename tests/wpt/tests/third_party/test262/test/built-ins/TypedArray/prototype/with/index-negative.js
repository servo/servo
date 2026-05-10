// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  %TypedArray%.prototype.with adds length to index if it's negative.
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
  var arr = new TA(makeCtorArg([0, 1, 2]));

  assert.compareArray(arr.with(-1, 4), [0, 1, 4]);
  assert.compareArray(arr.with(-3, 4), [4, 1, 2]);
  // -0 is not negative.
  assert.compareArray(arr.with(-0, 4), [4, 1, 2]);
});
