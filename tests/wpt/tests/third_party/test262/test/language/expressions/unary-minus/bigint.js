// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Unary minus for BigInt values
esid: sec-numeric-types-bigint-unaryMinus
info: |
  BigInt::unaryMinus (x)

  The abstract operation BigInt::unaryMinus with an argument x of BigInt type returns the result of negating x.

  Note: There is only one 0n value; -0n is the same as 0n.

features: [BigInt]
---*/

assert.sameValue(-0n, 0n, "-0n === 0n");
assert.sameValue(-(0n), 0n, "-(0n) === 0n");
assert.notSameValue(-1n, 1n, "-1n !== 1n");
assert.sameValue(-(1n), -1n, "-(1n) === -1n");
assert.notSameValue(-(1n), 1n, "-(1n) !== 1n");
assert.sameValue(-(-1n), 1n, "-(-1n) === 1n");
assert.notSameValue(-(-1n), -1n, "-(-1n) !== -1n");
assert.sameValue(- - 1n, 1n, "- - 1n === 1n");
assert.notSameValue(- - 1n, -1n, "- - 1n !== -1n");
assert.sameValue(
  -(0x1fffffffffffff01n), -0x1fffffffffffff01n,
  "-(0x1fffffffffffff01n) === -0x1fffffffffffff01n");
assert.notSameValue(
  -(0x1fffffffffffff01n), 0x1fffffffffffff01n,
  "-(0x1fffffffffffff01n) !== 0x1fffffffffffff01n");
assert.notSameValue(
  -(0x1fffffffffffff01n), -0x1fffffffffffff00n,
  "-(0x1fffffffffffff01n) !== -0x1fffffffffffff00n");
