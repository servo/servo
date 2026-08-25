// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tosorted
description: >
  Array.prototype.toSorted creates an empty array if the this value .length is not a positive integer.
info: |
  Array.prototype.toSorted ( compareFn )

  ...
  3. Let len be ? LengthOfArrayLike(O).
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

assert.compareArray(Array.prototype.toSorted.call({ length: -2 }), []);
assert.compareArray(Array.prototype.toSorted.call({ length: "dog" }), []);
assert.compareArray(Array.prototype.toSorted.call({ length: NaN }), []);
