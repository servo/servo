// Copyright (C) 2022 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: BigInt constructor only coerces its input once
esid: sec-bigint-constructor-number-value
info: |
  BigInt ( value )
    1. If NewTarget is not undefined, throw a TypeError exception.
    2. Let prim be ? ToPrimitive(value, number).
    3. If Type(prim) is Number, return ? NumberToBigInt(prim).
    4. Otherwise, return ? ToBigInt(prim).
features: [BigInt]
---*/

var first = true;
var v = {
  [Symbol.toPrimitive]: function() {
    if (first) {
      first = false;
      return "42";
    }
    throw new Test262Error("Symbol.toPrimitive should only be invoked once");
  },
};

assert.sameValue(BigInt(v), 42n, "BigInt constructor should use the post-ToPrimitive value as the argument to ToBigInt");
