// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced does not mutate its this value
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = [2, 0, 1];
arr.toSpliced(0, 0, -1);

assert.compareArray(arr, [2, 0, 1]);
assert.notSameValue(arr.toSpliced(0, 0, -1), arr);
assert.notSameValue(arr.toSpliced(0, 1, -1), arr);
