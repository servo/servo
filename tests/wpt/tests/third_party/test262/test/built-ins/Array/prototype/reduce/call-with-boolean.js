// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce applied to boolean primitive
---*/

assert.sameValue(
  Array.prototype.reduce.call(true, () => {}, -1),
  -1,
  'Array.prototype.reduce.call(true, () => {}, -1) must return -1'
);
assert.sameValue(
  Array.prototype.reduce.call(false, () => {}, -1),
  -1,
  'Array.prototype.reduce.call(false, () => {}, -1) must return -1'
);
