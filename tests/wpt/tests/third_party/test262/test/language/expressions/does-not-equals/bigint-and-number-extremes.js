// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Non-strict inequality comparison of BigInt and large Number values
esid: sec-abstract-equality-comparison
info: |
  12. If Type(x) is BigInt and Type(y) is Number, or if Type(x) is Number and Type(y) is BigInt,
    b. If the mathematical value of x is equal to the mathematical value of y, return true, otherwise return false.

features: [BigInt]
---*/
assert.sameValue(1n != Number.MAX_VALUE, true, 'The result of (1n != Number.MAX_VALUE) is true');
assert.sameValue(Number.MAX_VALUE != 1n, true, 'The result of (Number.MAX_VALUE != 1n) is true');
assert.sameValue(1n != -Number.MAX_VALUE, true, 'The result of (1n != -Number.MAX_VALUE) is true');
assert.sameValue(-Number.MAX_VALUE != 1n, true, 'The result of (-Number.MAX_VALUE != 1n) is true');

assert.sameValue(
  0xfffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn != Number.MAX_VALUE,
  true,
  'The result of (0xfffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn != Number.MAX_VALUE) is true'
);

assert.sameValue(
  Number.MAX_VALUE != 0xfffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn,
  true,
  'The result of (Number.MAX_VALUE != 0xfffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn) is true'
);

assert.sameValue(
  0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000n != Number.MAX_VALUE,
  false,
  'The result of (0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000n != Number.MAX_VALUE) is false'
);

assert.sameValue(
  Number.MAX_VALUE != 0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000n,
  false,
  'The result of (Number.MAX_VALUE != 0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000n) is false'
);

assert.sameValue(
  0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001n != Number.MAX_VALUE,
  true,
  'The result of (0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001n != Number.MAX_VALUE) is true'
);

assert.sameValue(
  Number.MAX_VALUE != 0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001n,
  true,
  'The result of (Number.MAX_VALUE != 0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001n) is true'
);
