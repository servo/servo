// Copyright (C) 2025 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tosorted
description: >
  Array.prototype.toSorted performs CompareArrayElements that sorts by the
  provided comparator function
info: |
  Array.prototype.toSorted ( compareFn )

  ...
  5. Let SortCompare be a new Abstract Closure with parameters (x, y) that
     captures comparator and performs the following steps when called:
    a. Return ? CompareArrayElements(x, y, comparator).
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

function numericCompare(a, b) {
    return a - b;
}

assert.compareArray([1, 2, 3, 4].toSorted(numericCompare), [1, 2, 3, 4]);
assert.compareArray([4, 3, 2, 1].toSorted(numericCompare), [1, 2, 3, 4]);
assert.compareArray(
    [333, 33, 3, 222, 22, 2, 111, 11, 1].toSorted(numericCompare),
    [1, 2, 3, 11, 22, 33, 111, 222, 333]);

function reverseNumericCompare(a, b) {
    return b - a;
}

assert.compareArray([1, 2, 3, 4].toSorted(reverseNumericCompare), [4, 3, 2, 1]);
assert.compareArray([4, 3, 2, 1].toSorted(reverseNumericCompare), [4, 3, 2, 1]);
assert.compareArray(
    [333, 33, 3, 222, 22, 2, 111, 11, 1].toSorted(reverseNumericCompare),
    [333, 222, 111, 33, 22, 11, 3, 2, 1]);
