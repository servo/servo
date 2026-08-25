// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Comparisons of BigInt and non-finite Number values
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
features: [BigInt]
---*/
assert.sameValue(1n < Infinity, true, 'The result of (1n < Infinity) is true');
assert.sameValue(Infinity < 1n, false, 'The result of (Infinity < 1n) is false');
assert.sameValue(-1n < Infinity, true, 'The result of (-1n < Infinity) is true');
assert.sameValue(Infinity < -1n, false, 'The result of (Infinity < -1n) is false');
assert.sameValue(1n < -Infinity, false, 'The result of (1n < -Infinity) is false');
assert.sameValue(-Infinity < 1n, true, 'The result of (-Infinity < 1n) is true');
assert.sameValue(-1n < -Infinity, false, 'The result of (-1n < -Infinity) is false');
assert.sameValue(-Infinity < -1n, true, 'The result of (-Infinity < -1n) is true');
assert.sameValue(0n < NaN, false, 'The result of (0n < NaN) is false');
assert.sameValue(NaN < 0n, false, 'The result of (NaN < 0n) is false');
