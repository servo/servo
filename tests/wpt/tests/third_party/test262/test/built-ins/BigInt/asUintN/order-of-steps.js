// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-bigint.asuintn
description: BigInt.asUintN order of parameter type coercion
info: |
  BigInt.asUintN ( bits, bigint )

  1. Let bits be ? ToIndex(bits).
  2. Let bigint ? ToBigInt(bigint).

features: [BigInt]
---*/

var i = 0;
var bits = {
  valueOf() {
    assert.sameValue(i, 0);
    i++;
    return 0;
  }
};
var bigint = {
  valueOf() {
    assert.sameValue(i, 1);
    i++;
    return 0n;
  }
};

BigInt.asUintN(bits, bigint);
assert.sameValue(i, 2);
