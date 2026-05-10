// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tosorted
description: >
  Array.prototype.toSorted converts the this value length to a number.
info: |
  Array.prototype.toSorted ( compareFn )

  ...
  3. Let len be ? LengthOfArrayLike(O).
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

assert.compareArray(Array.prototype.toSorted.call({ length: "2", 0: 4, 1: 0, 2: 1 }), [0, 4]);

var arrayLike = {
  length: {
    valueOf: () => 2
  },
  0: 4,
  1: 0,
  2: 1,
};

assert.compareArray(Array.prototype.toSorted.call(arrayLike), [0, 4]);
