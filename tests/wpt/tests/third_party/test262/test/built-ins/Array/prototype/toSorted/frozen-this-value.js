// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tosorted
description: >
  Array.prototype.toSorted works on frozen objects
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = Object.freeze([2, 0, 1]);
var result = arr.toSorted();
assert.compareArray(result, [0, 1, 2]);

var arrayLike = Object.freeze({ length: 3, 0: 2, 1: 0, 2: 1 });
result = Array.prototype.toSorted.call(arrayLike);
assert.compareArray(result, [0, 1, 2]);
