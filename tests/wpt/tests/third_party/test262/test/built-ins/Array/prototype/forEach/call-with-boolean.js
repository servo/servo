// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach applied to boolean primitive
---*/

assert.sameValue(
  Array.prototype.forEach.call(true, () => {}),
  undefined,
  'Array.prototype.forEach.call(true, () => {}) must return undefined'
);
assert.sameValue(
  Array.prototype.forEach.call(false, () => {}),
  undefined,
  'Array.prototype.forEach.call(false, () => {}) must return undefined'
);
