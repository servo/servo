// Copyright (C) 2025 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tosorted
description: >
  Array.prototype.toSorted performs CompareArrayElements that sorts by string
  by default
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

assert.compareArray([1, 2, 3, 4].toSorted(), [1, 2, 3, 4]);
assert.compareArray([4, 3, 2, 1].toSorted(), [1, 2, 3, 4]);
assert.compareArray(['a', 2, 1, 'z'].toSorted(), [1, 2, 'a', 'z']);

assert.compareArray(
    [333, 33, 3, 222, 22, 2, 111, 11, 1].toSorted(),
    [1, 11, 111, 2, 22, 222, 3, 33, 333]);
