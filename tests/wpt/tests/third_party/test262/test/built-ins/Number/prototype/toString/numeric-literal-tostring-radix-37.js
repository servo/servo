// Copyright 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.tostring
description: >
  If radixNumber < 2 or radixNumber > 36, throw a RangeError exception.
---*/

assert.throws(RangeError, () => {
  0..toString(37);
});
assert.throws(RangeError, () => {
  1..toString(37);
});
assert.throws(RangeError, () => {
  NaN.toString(37);
});
assert.throws(RangeError, () => {
  Infinity.toString(37);
});
