// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced converts the this value length to a number.
info: |
  Array.prototype.toSpliced ( start, deleteCount, ...items )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

assert.compareArray(Array.prototype.toSpliced.call({ length: "2", 0: 0, 1: 1, 2: 2 }, 0, 0), [0, 1]);

var arrayLike = {
  length: {
    valueOf: () => 2
  },
  0: 0,
  1: 1,
  2: 2,
};

assert.compareArray(Array.prototype.toSpliced.call(arrayLike, 0, 0), [0, 1]);
