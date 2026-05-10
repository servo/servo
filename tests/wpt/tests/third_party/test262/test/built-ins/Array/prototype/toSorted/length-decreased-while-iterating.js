// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tosorted
description: >
  Array.prototype.toSorted caches the length getting the array elements.
info: |
  Array.prototype.toSorted ( compareFn )

  ...
  3. Let len be ? LengthOfArrayLike(O).
  ...
  6. Let sortedList be ? SortIndexedProperties(obj, len, SortCompare, false).
  ...

features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = [5, 1, 4, 6, 3];
Array.prototype[3] = 2;

Object.defineProperty(arr, "2", {
  get() {
    arr.length = 1;
    return 4;
  }
});

assert.compareArray(arr.toSorted(), [1, 2, 4, 5, undefined]);
