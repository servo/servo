// Copyright (C) 2017 Josh Wolfe. All rights reserved.
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
assert.sameValue(1n > Number.MAX_VALUE, false, 'The result of (1n > Number.MAX_VALUE) is false');
assert.sameValue(Number.MAX_VALUE > 1n, true, 'The result of (Number.MAX_VALUE > 1n) is true');
assert.sameValue(1n > -Number.MAX_VALUE, true, 'The result of (1n > -Number.MAX_VALUE) is true');
assert.sameValue(-Number.MAX_VALUE > 1n, false, 'The result of (-Number.MAX_VALUE > 1n) is false');

assert.sameValue(
  0xfffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn > Number.MAX_VALUE,
  false,
  'The result of (0xfffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn > Number.MAX_VALUE) is false'
);

assert.sameValue(
  Number.MAX_VALUE > 0xfffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn,
  true,
  'The result of (Number.MAX_VALUE > 0xfffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn) is true'
);

assert.sameValue(
  0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001n > Number.MAX_VALUE,
  true,
  'The result of (0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001n > Number.MAX_VALUE) is true'
);

assert.sameValue(
  Number.MAX_VALUE > 0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001n,
  false,
  'The result of (Number.MAX_VALUE > 0xfffffffffffff800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001n) is false'
);
