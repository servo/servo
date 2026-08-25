// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tosorted
description: >
  Array.prototype.toSorted returns a new array even if it has zero or one elements
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var zero = [];
var zeroReversed = zero.toSorted();
assert.notSameValue(zero, zeroReversed);
assert.compareArray(zero, zeroReversed);

var one = [1];
var oneReversed = one.toSorted();
assert.notSameValue(one, oneReversed);
assert.compareArray(one, oneReversed);
