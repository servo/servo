// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Left shift for BigInt values
esid: sec-numeric-types-bigint-leftShift
info: |
  BigInt::leftShift (x, y)

  The abstract operation BigInt::leftShift with two arguments x and y of BigInt:

  1. If y < 0,
    a. Return a BigInt representing x divided by 2-y, rounding down to the nearest integer, including for negative numbers.
  2. Return a BigInt representing x multiplied by 2y.

  NOTE: Semantics here should be equivalent to a bitwise shift, treating the BigInt as an infinite length string of binary two's complement digits.

features: [BigInt]
---*/

assert.sameValue(0n << 0n, 0n, "0n << 0n === 0n");
assert.sameValue(0b101n << 1n, 0b1010n, "0b101n << 1n === 0b1010n");
assert.sameValue(0b101n << 2n, 0b10100n, "0b101n << 2n === 0b10100n");
assert.sameValue(0b101n << 3n, 0b101000n, "0b101n << 3n === 0b101000n");
assert.sameValue(0b101n << -1n, 0b10n, "0b101n << -1n === 0b10n");
assert.sameValue(0b101n << -2n, 1n, "0b101n << -2n === 1n");
assert.sameValue(0b101n << -3n, 0n, "0b101n << -3n === 0n");
assert.sameValue(0n << 128n, 0n, "0n << 128n === 0n");
assert.sameValue(0n << -128n, 0n, "0n << -128n === 0n");
assert.sameValue(0x246n << 0n, 0x246n, "0x246n << 0n === 0x246n");
assert.sameValue(0x246n << 127n, 0x12300000000000000000000000000000000n, "0x246n << 127n === 0x12300000000000000000000000000000000n");
assert.sameValue(0x246n << 128n, 0x24600000000000000000000000000000000n, "0x246n << 128n === 0x24600000000000000000000000000000000n");
assert.sameValue(0x246n << 129n, 0x48c00000000000000000000000000000000n, "0x246n << 129n === 0x48c00000000000000000000000000000000n");
assert.sameValue(0x246n << -128n, 0n, "0x246n << -128n === 0n");
assert.sameValue(
  0x123456789abcdef0fedcba9876543212345678n << 64n, 0x123456789abcdef0fedcba98765432123456780000000000000000n,
  "0x123456789abcdef0fedcba9876543212345678n << 64n === 0x123456789abcdef0fedcba98765432123456780000000000000000n");
assert.sameValue(
  0x123456789abcdef0fedcba9876543212345678n << 32n, 0x123456789abcdef0fedcba987654321234567800000000n,
  "0x123456789abcdef0fedcba9876543212345678n << 32n === 0x123456789abcdef0fedcba987654321234567800000000n");
assert.sameValue(
  0x123456789abcdef0fedcba9876543212345678n << 16n, 0x123456789abcdef0fedcba98765432123456780000n,
  "0x123456789abcdef0fedcba9876543212345678n << 16n === 0x123456789abcdef0fedcba98765432123456780000n");
assert.sameValue(
  0x123456789abcdef0fedcba9876543212345678n << 0n, 0x123456789abcdef0fedcba9876543212345678n,
  "0x123456789abcdef0fedcba9876543212345678n << 0n === 0x123456789abcdef0fedcba9876543212345678n");
assert.sameValue(
  0x123456789abcdef0fedcba9876543212345678n << -16n, 0x123456789abcdef0fedcba987654321234n,
  "0x123456789abcdef0fedcba9876543212345678n << -16n === 0x123456789abcdef0fedcba987654321234n");
assert.sameValue(
  0x123456789abcdef0fedcba9876543212345678n << -32n, 0x123456789abcdef0fedcba98765432n,
  "0x123456789abcdef0fedcba9876543212345678n << -32n === 0x123456789abcdef0fedcba98765432n");
assert.sameValue(
  0x123456789abcdef0fedcba9876543212345678n << -64n, 0x123456789abcdef0fedcban,
  "0x123456789abcdef0fedcba9876543212345678n << -64n === 0x123456789abcdef0fedcban");
assert.sameValue(
  0x123456789abcdef0fedcba9876543212345678n << -127n, 0x2468acn,
  "0x123456789abcdef0fedcba9876543212345678n << -127n === 0x2468acn");
assert.sameValue(
  0x123456789abcdef0fedcba9876543212345678n << -128n, 0x123456n,
  "0x123456789abcdef0fedcba9876543212345678n << -128n === 0x123456n");
assert.sameValue(
  0x123456789abcdef0fedcba9876543212345678n << -129n, 0x91a2bn,
  "0x123456789abcdef0fedcba9876543212345678n << -129n === 0x91a2bn");
assert.sameValue(-5n << 1n, -0xan, "-5n << 1n === -0xan");
assert.sameValue(-5n << 2n, -0x14n, "-5n << 2n === -0x14n");
assert.sameValue(-5n << 3n, -0x28n, "-5n << 3n === -0x28n");
assert.sameValue(-5n << -1n, -3n, "-5n << -1n === -3n");
assert.sameValue(-5n << -2n, -2n, "-5n << -2n === -2n");
assert.sameValue(-5n << -3n, -1n, "-5n << -3n === -1n");
assert.sameValue(-1n << 128n, -0x100000000000000000000000000000000n, "-1n << 128n === -0x100000000000000000000000000000000n");
assert.sameValue(-1n << 0n, -1n, "-1n << 0n === -1n");
assert.sameValue(-1n << -128n, -1n, "-1n << -128n === -1n");
assert.sameValue(-0x246n << 0n, -0x246n, "-0x246n << 0n === -0x246n");
assert.sameValue(-0x246n << 127n, -0x12300000000000000000000000000000000n, "-0x246n << 127n === -0x12300000000000000000000000000000000n");
assert.sameValue(-0x246n << 128n, -0x24600000000000000000000000000000000n, "-0x246n << 128n === -0x24600000000000000000000000000000000n");
assert.sameValue(-0x246n << 129n, -0x48c00000000000000000000000000000000n, "-0x246n << 129n === -0x48c00000000000000000000000000000000n");
assert.sameValue(-0x246n << -128n, -1n, "-0x246n << -128n === -1n");
assert.sameValue(
  -0x123456789abcdef0fedcba9876543212345678n << 64n, -0x123456789abcdef0fedcba98765432123456780000000000000000n,
  "-0x123456789abcdef0fedcba9876543212345678n << 64n === -0x123456789abcdef0fedcba98765432123456780000000000000000n");
assert.sameValue(
  -0x123456789abcdef0fedcba9876543212345678n << 32n, -0x123456789abcdef0fedcba987654321234567800000000n,
  "-0x123456789abcdef0fedcba9876543212345678n << 32n === -0x123456789abcdef0fedcba987654321234567800000000n");
assert.sameValue(
  -0x123456789abcdef0fedcba9876543212345678n << 16n, -0x123456789abcdef0fedcba98765432123456780000n,
  "-0x123456789abcdef0fedcba9876543212345678n << 16n === -0x123456789abcdef0fedcba98765432123456780000n");
assert.sameValue(
  -0x123456789abcdef0fedcba9876543212345678n << 0n, -0x123456789abcdef0fedcba9876543212345678n,
  "-0x123456789abcdef0fedcba9876543212345678n << 0n === -0x123456789abcdef0fedcba9876543212345678n");
assert.sameValue(
  -0x123456789abcdef0fedcba9876543212345678n << -16n, -0x123456789abcdef0fedcba987654321235n,
  "-0x123456789abcdef0fedcba9876543212345678n << -16n === -0x123456789abcdef0fedcba987654321235n");
assert.sameValue(
  -0x123456789abcdef0fedcba9876543212345678n << -32n, -0x123456789abcdef0fedcba98765433n,
  "-0x123456789abcdef0fedcba9876543212345678n << -32n === -0x123456789abcdef0fedcba98765433n");
assert.sameValue(
  -0x123456789abcdef0fedcba9876543212345678n << -64n, -0x123456789abcdef0fedcbbn,
  "-0x123456789abcdef0fedcba9876543212345678n << -64n === -0x123456789abcdef0fedcbbn");
assert.sameValue(
  -0x123456789abcdef0fedcba9876543212345678n << -127n, -0x2468adn,
  "-0x123456789abcdef0fedcba9876543212345678n << -127n === -0x2468adn");
assert.sameValue(
  -0x123456789abcdef0fedcba9876543212345678n << -128n, -0x123457n,
  "-0x123456789abcdef0fedcba9876543212345678n << -128n === -0x123457n");
assert.sameValue(
  -0x123456789abcdef0fedcba9876543212345678n << -129n, -0x91a2cn,
  "-0x123456789abcdef0fedcba9876543212345678n << -129n === -0x91a2cn");
