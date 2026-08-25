// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.toreversed
description: >
  Array.prototype.toReversed creates an empty array if the this value .length is not a positive integer.
info: |
  Array.prototype.toReversed ( )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

assert.compareArray(Array.prototype.toReversed.call({ length: -2 }), []);
assert.compareArray(Array.prototype.toReversed.call({ length: "dog" }), []);
assert.compareArray(Array.prototype.toReversed.call({ length: NaN }), []);
