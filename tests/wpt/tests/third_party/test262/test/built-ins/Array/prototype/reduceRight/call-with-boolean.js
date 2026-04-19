// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: Array.prototype.reduceRight applied to boolean primitive
---*/

assert.sameValue(
  Array.prototype.reduceRight.call(true, () => {}, -1),
  -1,
  'Array.prototype.reduceRight.call(true, () => {}, -1) must return -1'
);
assert.sameValue(
  Array.prototype.reduceRight.call(false, () => {}, -1),
  -1,
  'Array.prototype.reduceRight.call(false, () => {}, -1) must return -1'
);
