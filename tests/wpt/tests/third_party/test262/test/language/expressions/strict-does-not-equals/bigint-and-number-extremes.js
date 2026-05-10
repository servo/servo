// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Strict inequality comparison of BigInt and large Number values
esid: sec-strict-equality-comparison
info: |
  1. If Type(x) is different from Type(y), return false.

features: [BigInt]
---*/
assert.sameValue(1n !== Number.MAX_VALUE, true, 'The result of (1n !== Number.MAX_VALUE) is true');
assert.sameValue(Number.MAX_VALUE !== 1n, true, 'The result of (Number.MAX_VALUE !== 1n) is true');

assert.sameValue(
  1n !== -Number.MAX_VALUE,
  true,
  'The result of (1n !== -Number.MAX_VALUE) is true'
);

assert.sameValue(
  -Number.MAX_VALUE !== 1n,
  true,
  'The result of (-Number.MAX_VALUE !== 1n) is true'
);

assert.sameValue(
  0xfffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn !== Number.MAX_VALUE,
  true,
  'The result of (0xfffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn !== Number.MAX_VALUE) is true'
);

assert.sameValue(
  Number.MAX_VALUE !== 0xfffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn,
  true,
  'The result of (Number.MAX_VALUE !== 0xfffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn) is true'
);

assert.sameValue(
  0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000n !== Number.MAX_VALUE,
  true,
  'The result of (0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000n !== Number.MAX_VALUE) is true'
);

assert.sameValue(
  Number.MAX_VALUE !== 0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000n,
  true,
  'The result of (Number.MAX_VALUE !== 0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000n) is true'
);

assert.sameValue(
  0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001n !== Number.MAX_VALUE,
  true,
  'The result of (0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001n !== Number.MAX_VALUE) is true'
);

assert.sameValue(
  Number.MAX_VALUE !== 0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001n,
  true,
  'The result of (Number.MAX_VALUE !== 0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001n) is true'
);
