// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced works on frozen objects
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = Object.freeze([2, 0, 1]);
var result = arr.toSpliced();
assert.compareArray(result, [2, 0, 1]);

var arrayLike = Object.freeze({ length: 3, 0: 0, 1: 1, 2: 2 });

assert.compareArray(Array.prototype.toSpliced.call(arrayLike, 1, 1, 4, 5), [0, 4, 5, 2]);
