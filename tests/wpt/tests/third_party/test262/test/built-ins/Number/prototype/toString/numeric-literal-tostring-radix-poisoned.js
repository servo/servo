// Copyright 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.tostring
description: >
  Else, let radixNumber be ? ToInteger(radix).
---*/

var poisoned = {
  valueOf() {
    throw new Test262Error();
  }
};
assert.throws(Test262Error, () => {
  0..toString(poisoned);
});
assert.throws(Test262Error, () => {
  1..toString(poisoned);
});
assert.throws(Test262Error, () => {
  NaN.toString(poisoned);
});
assert.throws(Test262Error, () => {
  Infinity.toString(poisoned);
});
