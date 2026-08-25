// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Non-strict inequality comparison of BigInt and Number values
esid: sec-abstract-equality-comparison
info: |
  12. If Type(x) is BigInt and Type(y) is Number, or if Type(x) is Number and Type(y) is BigInt,
    b. If the mathematical value of x is equal to the mathematical value of y, return true, otherwise return false.

features: [BigInt]
---*/
assert.sameValue(0n != 0, false, 'The result of (0n != 0) is false');
assert.sameValue(0 != 0n, false, 'The result of (0 != 0n) is false');
assert.sameValue(0n != -0, false, 'The result of (0n != -0) is false');
assert.sameValue(-0 != 0n, false, 'The result of (-0 != 0n) is false');
assert.sameValue(0n != 0.000000000001, true, 'The result of (0n != 0.000000000001) is true');
assert.sameValue(0.000000000001 != 0n, true, 'The result of (0.000000000001 != 0n) is true');
assert.sameValue(0n != 1, true, 'The result of (0n != 1) is true');
assert.sameValue(1 != 0n, true, 'The result of (1 != 0n) is true');
assert.sameValue(1n != 0, true, 'The result of (1n != 0) is true');
assert.sameValue(0 != 1n, true, 'The result of (0 != 1n) is true');
assert.sameValue(1n != 0.999999999999, true, 'The result of (1n != 0.999999999999) is true');
assert.sameValue(0.999999999999 != 1n, true, 'The result of (0.999999999999 != 1n) is true');
assert.sameValue(1n != 1, false, 'The result of (1n != 1) is false');
assert.sameValue(1 != 1n, false, 'The result of (1 != 1n) is false');
assert.sameValue(0n != Number.MIN_VALUE, true, 'The result of (0n != Number.MIN_VALUE) is true');
assert.sameValue(Number.MIN_VALUE != 0n, true, 'The result of (Number.MIN_VALUE != 0n) is true');
assert.sameValue(0n != -Number.MIN_VALUE, true, 'The result of (0n != -Number.MIN_VALUE) is true');
assert.sameValue(-Number.MIN_VALUE != 0n, true, 'The result of (-Number.MIN_VALUE != 0n) is true');

assert.sameValue(
  -10n != Number.MIN_VALUE,
  true,
  'The result of (-10n != Number.MIN_VALUE) is true'
);

assert.sameValue(
  Number.MIN_VALUE != -10n,
  true,
  'The result of (Number.MIN_VALUE != -10n) is true'
);
