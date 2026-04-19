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

var arr = [5, 0, 3];
Object.defineProperty(arr, "0", {
  get() {
    arr.push(1);
    return 5;
  }
});

assert.compareArray(arr.toSorted(), [0, 3, 5]);
