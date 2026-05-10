// Copyright (C) 2017 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws exception in BigIntConstructor if toString throws
esid: sec-bigint-constructor-number-value
info: |
  1. If NewTarget is not undefined, throw a TypeError exception.
  2. Let prim be ? ToPrimitive(value, hint Number).
  3. If Type(prim) is Number, return ? NumberToBigInt(prim).
  4. Otherwise, return ? ToBigInt(value).
features: [BigInt]
---*/

assert.throws(Test262Error, function() {
  BigInt({
    toString: function() {
      throw new Test262Error();
    }
  });
});
