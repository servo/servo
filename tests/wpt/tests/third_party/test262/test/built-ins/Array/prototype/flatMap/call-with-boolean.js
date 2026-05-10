// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.flatmap
description: Array.prototype.flatMap applied to boolean primitive
includes: [compareArray.js]
---*/

assert.compareArray(
  Array.prototype.flatMap.call(true, () => {}),
  [],
  'Array.prototype.flatMap.call(true, () => {}) must return []'
);
assert.compareArray(
  Array.prototype.flatMap.call(false, () => {}),
  [],
  'Array.prototype.flatMap.call(false, () => {}) must return []'
);
