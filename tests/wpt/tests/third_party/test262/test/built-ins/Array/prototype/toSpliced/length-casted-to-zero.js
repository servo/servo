// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced treats its `this` value's `length` property as zero if the
  property's value is not a positive integer.
info: |
  Array.prototype.toSpliced ( start, deleteCount, ...items )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

assert.compareArray(Array.prototype.toSpliced.call({ length: -2 }, 0, 0, 2, 3), [2, 3]);
assert.compareArray(Array.prototype.toSpliced.call({ length: "dog" }, 0, 0, 2, 3), [2, 3]);
assert.compareArray(Array.prototype.toSpliced.call({ length: NaN }, 0, 0, 2, 3), [2, 3]);
