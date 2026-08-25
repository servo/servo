// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.findindex
description: Array.prototype.findIndex applied to boolean primitive
---*/

assert.sameValue(
  Array.prototype.findIndex.call(true, () => {}),
  -1,
  'Array.prototype.findIndex.call(true, () => {}) must return -1'
);
assert.sameValue(
  Array.prototype.findIndex.call(false, () => {}),
  -1,
  'Array.prototype.findIndex.call(false, () => {}) must return -1'
);
