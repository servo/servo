// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tosorted
description: >
  Array.prototype.toSorted does not preserve holes in the array
info: |
  Array.prototype.toSorted ( compareFn )

  ...
  8. Repeat, while j < len,
    a. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(j)), sortedList[j]).
    b. Set j to j + 1.
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = [3, /* hole */, 4, /* hole */, 1];
Array.prototype[3] = 2;

var sorted = arr.toSorted();
assert.compareArray(sorted, [1, 2, 3, 4, undefined]);
assert(sorted.hasOwnProperty(4));
