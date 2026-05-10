// Copyright (C) 2025 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.tosorted
description: >
  %TypedArray%.prototype.toSorted performs CompareTypedArrayElements that sorts
  numerically by default
info: |
  %TypedArray%.prototype.toSorted ( compareFn )

  ...
  7. Let SortCompare be a new Abstract Closure with parameters (x, y) that
     captures comparator and performs the following steps when called:
    a. Return ? CompareTypedArrayElements(x, y, comparator).
  ...
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray, change-array-by-copy]
---*/

testWithTypedArrayConstructors(TA => {
    assert.compareArray(new TA([4, 2, 1, 3]).toSorted(), [1, 2, 3, 4]);
    assert.compareArray(
        new TA([111, 33, 22, 11, 3, 2, 1]).toSorted(),
        [1, 2, 3, 11, 22, 33, 111]);
});
