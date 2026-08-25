// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.with
description: >
  Array.prototype.with does not mutate its this value
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var arr = [0, 1, 2];
arr.with(1, 3);

assert.compareArray(arr, [0, 1, 2]);
assert.notSameValue(arr.with(1, 3), arr);
assert.notSameValue(arr.with(1, 1), arr);
