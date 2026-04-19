// Copyright 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.tostring
description: Direct toString on BigInt prototype
info: |
  BigInt.prototype.toString ( [ radix ] )

  Let x be ? thisBigIntValue(this value).

  Properties of the BigInt Prototype Object

  The BigInt prototype is not a BigInt object; it does not have a
  [[BigIntData]] internal slot.
features: [BigInt]
---*/
assert.sameValue(typeof BigInt, 'function');

assert.throws(TypeError, function() {
  BigInt.prototype.toString(1);
});
