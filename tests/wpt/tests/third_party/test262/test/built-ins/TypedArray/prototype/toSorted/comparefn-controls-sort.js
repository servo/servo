// Copyright (C) 2025 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.tosorted
description: >
  %TypedArray%.prototype.toSorted performs CompareTypedArrayElements that sorts
  by the provided comparator function
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

function reverseNumericCompare(a, b) {
    return b - a;
}

testWithTypedArrayConstructors(TA => {
    assert.compareArray(
        new TA([1, 2, 3, 4]).toSorted(reverseNumericCompare),
        [4, 3, 2, 1]);
    assert.compareArray(
        new TA([4, 3, 2, 1]).toSorted(reverseNumericCompare),
        [4, 3, 2, 1]);
    assert.compareArray(
        new TA([33, 3, 22, 2, 111, 11, 1]).toSorted(reverseNumericCompare),
        [111, 33, 22, 11, 3, 2, 1]);
});
