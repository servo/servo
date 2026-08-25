// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Comparisons of BigInt and Number values
esid: sec-abstract-relational-comparison
info: |
  ...
  3. If both px and py are Strings, then
    ...
  4. Else,
    a. Let nx be ? ToNumeric(px). Because px and py are primitive values evaluation order is not important.
    b. Let ny be ? ToNumeric(py).
    c. If Type(nx) is Type(ny), return ? Type(nx)::lessThan(nx, ny).
    d. Assert: Type(nx) is BigInt and Type(ny) is Number, or if Type(nx) is Number and Type(ny) is BigInt.
    e. If x or y are any of NaN, return undefined.
    f. If x is -∞, or y is +∞, return true.
    g. If x is +∞, or y is -∞, return false.
    h. If the mathematical value of nx is less than the mathematical value of ny, return true, otherwise return false.
features: [BigInt]
---*/
assert.sameValue(0n <= 0, true, 'The result of (0n <= 0) is true');
assert.sameValue(0 <= 0n, true, 'The result of (0 <= 0n) is true');
assert.sameValue(0n <= -0, true, 'The result of (0n <= -0) is true');
assert.sameValue(-0 <= 0n, true, 'The result of (-0 <= 0n) is true');
assert.sameValue(0n <= 0.000000000001, true, 'The result of (0n <= 0.000000000001) is true');
assert.sameValue(0.000000000001 <= 0n, false, 'The result of (0.000000000001 <= 0n) is false');
assert.sameValue(0n <= 1, true, 'The result of (0n <= 1) is true');
assert.sameValue(1 <= 0n, false, 'The result of (1 <= 0n) is false');
assert.sameValue(1n <= 0, false, 'The result of (1n <= 0) is false');
assert.sameValue(0 <= 1n, true, 'The result of (0 <= 1n) is true');
assert.sameValue(1n <= 0.999999999999, false, 'The result of (1n <= 0.999999999999) is false');
assert.sameValue(0.999999999999 <= 1n, true, 'The result of (0.999999999999 <= 1n) is true');
assert.sameValue(1n <= 1, true, 'The result of (1n <= 1) is true');
assert.sameValue(1 <= 1n, true, 'The result of (1 <= 1n) is true');
assert.sameValue(0n <= Number.MIN_VALUE, true, 'The result of (0n <= Number.MIN_VALUE) is true');
assert.sameValue(Number.MIN_VALUE <= 0n, false, 'The result of (Number.MIN_VALUE <= 0n) is false');

assert.sameValue(
  0n <= -Number.MIN_VALUE,
  false,
  'The result of (0n <= -Number.MIN_VALUE) is false'
);

assert.sameValue(-Number.MIN_VALUE <= 0n, true, 'The result of (-Number.MIN_VALUE <= 0n) is true');

assert.sameValue(
  -10n <= Number.MIN_VALUE,
  true,
  'The result of (-10n <= Number.MIN_VALUE) is true'
);

assert.sameValue(
  Number.MIN_VALUE <= -10n,
  false,
  'The result of (Number.MIN_VALUE <= -10n) is false'
);
