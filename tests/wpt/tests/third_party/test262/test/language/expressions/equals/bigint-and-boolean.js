// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Non-strict equality comparison of BigInt and Boolean values
esid: sec-abstract-equality-comparison
info: |
  8. If Type(x) is Boolean, return the result of the comparison ToNumber(x) == y.
  9. If Type(y) is Boolean, return the result of the comparison x == ToNumber(y).
  ...
  12. If Type(x) is BigInt and Type(y) is Number, or if Type(x) is Number and Type(y) is BigInt,
    ...
    b. If the mathematical value of x is equal to the mathematical value of y, return true, otherwise return false.

features: [BigInt]
---*/
assert.sameValue(-1n == false, false, 'The result of (-1n == false) is false');
assert.sameValue(false == -1n, false, 'The result of (false == -1n) is false');
assert.sameValue(-1n == true, false, 'The result of (-1n == true) is false');
assert.sameValue(true == -1n, false, 'The result of (true == -1n) is false');
assert.sameValue(0n == false, true, 'The result of (0n == false) is true');
assert.sameValue(false == 0n, true, 'The result of (false == 0n) is true');
assert.sameValue(0n == true, false, 'The result of (0n == true) is false');
assert.sameValue(true == 0n, false, 'The result of (true == 0n) is false');
assert.sameValue(1n == false, false, 'The result of (1n == false) is false');
assert.sameValue(false == 1n, false, 'The result of (false == 1n) is false');
assert.sameValue(1n == true, true, 'The result of (1n == true) is true');
assert.sameValue(true == 1n, true, 'The result of (true == 1n) is true');
assert.sameValue(2n == false, false, 'The result of (2n == false) is false');
assert.sameValue(false == 2n, false, 'The result of (false == 2n) is false');
assert.sameValue(2n == true, false, 'The result of (2n == true) is false');
assert.sameValue(true == 2n, false, 'The result of (true == 2n) is false');
