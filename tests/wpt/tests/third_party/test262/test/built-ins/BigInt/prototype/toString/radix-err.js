// Copyright 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.tostring
description: toString with invalid radix
info: |
  BigInt.prototype.toString ( [ radix ] )

  [...]
  4. Else, let radixNumber be ? ToInteger(radix).
  5. If radixNumber < 2 or radixNumber > 36, throw a RangeError
     exception.
features: [BigInt]
---*/

for (let r of [0, 1, 37, null]) {
  assert.throws(RangeError, function() {
    (0n).toString(r);
  }, "0, radix " + r);
  assert.throws(RangeError, function() {
    (-1n).toString(r);
  }, "-1, radix " + r);
  assert.throws(RangeError, function() {
    (1n).toString(r);
  }, "1, radix " + r);
}
