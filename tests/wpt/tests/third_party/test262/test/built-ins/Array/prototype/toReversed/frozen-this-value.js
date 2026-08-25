// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.toreversed
description: >
  Array.prototype.toReversed works on frozen objects
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = Object.freeze([0, 1, 2]);
var result = arr.toReversed();
assert.compareArray(result, [2, 1, 0]);

var arrayLike = Object.freeze({ length: 3, 0: 0, 1: 1, 2: 2 });
result = Array.prototype.toReversed.call(arrayLike);
assert.compareArray(result, [2, 1, 0]);
