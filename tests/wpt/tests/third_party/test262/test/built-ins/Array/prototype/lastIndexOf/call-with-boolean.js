// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf applied to boolean primitive
---*/

assert.sameValue(Array.prototype.lastIndexOf.call(true), -1, 'Array.prototype.lastIndexOf.call(true) must return -1');
assert.sameValue(
  Array.prototype.lastIndexOf.call(false),
  -1,
  'Array.prototype.lastIndexOf.call(false) must return -1'
);
