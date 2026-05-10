// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Bitwise OR for BigInt values
esid: sec-bitwise-op
info: |
  BitwiseOp(op, x, y)

  1. Let result be 0.
  2. Let shift be 0.
  3. Repeat, until (x = 0 or x = -1) and (y = 0 or y = -1),
    a. Let xDigit be x modulo 2.
    b. Let yDigit be y modulo 2.
    c. Let result be result + 2**shift * op(xDigit, yDigit)
    d. Let shift be shift + 1.
    e. Let x be (x - xDigit) / 2.
    f. Let y be (y - yDigit) / 2.
  4. If op(x modulo 2, y modulo 2) ≠ 0,
    a. Let result be result - 2**shift. NOTE: This extends the sign.
  5. Return result.

features: [BigInt]
---*/

assert.sameValue(0b00n | 0b00n, 0b00n, "0b00n | 0b00n === 0b00n");
assert.sameValue(0b00n | 0b01n, 0b01n, "0b00n | 0b01n === 0b01n");
assert.sameValue(0b01n | 0b00n, 0b01n, "0b01n | 0b00n === 0b01n");
assert.sameValue(0b00n | 0b10n, 0b10n, "0b00n | 0b10n === 0b10n");
assert.sameValue(0b10n | 0b00n, 0b10n, "0b10n | 0b00n === 0b10n");
assert.sameValue(0b00n | 0b11n, 0b11n, "0b00n | 0b11n === 0b11n");
assert.sameValue(0b11n | 0b00n, 0b11n, "0b11n | 0b00n === 0b11n");
assert.sameValue(0b01n | 0b01n, 0b01n, "0b01n | 0b01n === 0b01n");
assert.sameValue(0b01n | 0b10n, 0b11n, "0b01n | 0b10n === 0b11n");
assert.sameValue(0b10n | 0b01n, 0b11n, "0b10n | 0b01n === 0b11n");
assert.sameValue(0b01n | 0b11n, 0b11n, "0b01n | 0b11n === 0b11n");
assert.sameValue(0b11n | 0b01n, 0b11n, "0b11n | 0b01n === 0b11n");
assert.sameValue(0b10n | 0b10n, 0b10n, "0b10n | 0b10n === 0b10n");
assert.sameValue(0b10n | 0b11n, 0b11n, "0b10n | 0b11n === 0b11n");
assert.sameValue(0b11n | 0b10n, 0b11n, "0b11n | 0b10n === 0b11n");
assert.sameValue(0xffffffffn | 0n, 0xffffffffn, "0xffffffffn | 0n === 0xffffffffn");
assert.sameValue(0n | 0xffffffffn, 0xffffffffn, "0n | 0xffffffffn === 0xffffffffn");
assert.sameValue(0xffffffffn | 0xffffffffn, 0xffffffffn, "0xffffffffn | 0xffffffffn === 0xffffffffn");
assert.sameValue(0xffffffffffffffffn | 0n, 0xffffffffffffffffn, "0xffffffffffffffffn | 0n === 0xffffffffffffffffn");
assert.sameValue(0n | 0xffffffffffffffffn, 0xffffffffffffffffn, "0n | 0xffffffffffffffffn === 0xffffffffffffffffn");
assert.sameValue(0xffffffffffffffffn | 0xffffffffn, 0xffffffffffffffffn, "0xffffffffffffffffn | 0xffffffffn === 0xffffffffffffffffn");
assert.sameValue(0xffffffffn | 0xffffffffffffffffn, 0xffffffffffffffffn, "0xffffffffn | 0xffffffffffffffffn === 0xffffffffffffffffn");
assert.sameValue(
  0xffffffffffffffffn | 0xffffffffffffffffn, 0xffffffffffffffffn,
  "0xffffffffffffffffn | 0xffffffffffffffffn === 0xffffffffffffffffn");
assert.sameValue(
  0xbf2ed51ff75d380fd3be813ec6185780n | 0x4aabef2324cedff5387f1f65n, 0xbf2ed51fffffff2ff7fedffffe7f5fe5n,
  "0xbf2ed51ff75d380fd3be813ec6185780n | 0x4aabef2324cedff5387f1f65n === 0xbf2ed51fffffff2ff7fedffffe7f5fe5n");
assert.sameValue(
  0x4aabef2324cedff5387f1f65n | 0xbf2ed51ff75d380fd3be813ec6185780n, 0xbf2ed51fffffff2ff7fedffffe7f5fe5n,
  "0x4aabef2324cedff5387f1f65n | 0xbf2ed51ff75d380fd3be813ec6185780n === 0xbf2ed51fffffff2ff7fedffffe7f5fe5n");
assert.sameValue(0n | -1n, -1n, "0n | -1n === -1n");
assert.sameValue(-1n | 0n, -1n, "-1n | 0n === -1n");
assert.sameValue(0n | -2n, -2n, "0n | -2n === -2n");
assert.sameValue(-2n | 0n, -2n, "-2n | 0n === -2n");
assert.sameValue(1n | -2n, -1n, "1n | -2n === -1n");
assert.sameValue(-2n | 1n, -1n, "-2n | 1n === -1n");
assert.sameValue(2n | -2n, -2n, "2n | -2n === -2n");
assert.sameValue(-2n | 2n, -2n, "-2n | 2n === -2n");
assert.sameValue(2n | -3n, -1n, "2n | -3n === -1n");
assert.sameValue(-3n | 2n, -1n, "-3n | 2n === -1n");
assert.sameValue(-1n | -2n, -1n, "-1n | -2n === -1n");
assert.sameValue(-2n | -1n, -1n, "-2n | -1n === -1n");
assert.sameValue(-2n | -2n, -2n, "-2n | -2n === -2n");
assert.sameValue(-2n | -3n, -1n, "-2n | -3n === -1n");
assert.sameValue(-3n | -2n, -1n, "-3n | -2n === -1n");
assert.sameValue(0xffffffffn | -1n, -1n, "0xffffffffn | -1n === -1n");
assert.sameValue(-1n | 0xffffffffn, -1n, "-1n | 0xffffffffn === -1n");
assert.sameValue(0xffffffffffffffffn | -1n, -1n, "0xffffffffffffffffn | -1n === -1n");
assert.sameValue(-1n | 0xffffffffffffffffn, -1n, "-1n | 0xffffffffffffffffn === -1n");
assert.sameValue(
  0xbf2ed51ff75d380fd3be813ec6185780n | -0x4aabef2324cedff5387f1f65n, -0x8a2c72024405ec138670865n,
  "0xbf2ed51ff75d380fd3be813ec6185780n | -0x4aabef2324cedff5387f1f65n === -0x8a2c72024405ec138670865n");
assert.sameValue(
  -0x4aabef2324cedff5387f1f65n | 0xbf2ed51ff75d380fd3be813ec6185780n, -0x8a2c72024405ec138670865n,
  "-0x4aabef2324cedff5387f1f65n | 0xbf2ed51ff75d380fd3be813ec6185780n === -0x8a2c72024405ec138670865n");
assert.sameValue(
  -0xbf2ed51ff75d380fd3be813ec6185780n | 0x4aabef2324cedff5387f1f65n, -0xbf2ed51fb554100cd330000ac600401bn,
  "-0xbf2ed51ff75d380fd3be813ec6185780n | 0x4aabef2324cedff5387f1f65n === -0xbf2ed51fb554100cd330000ac600401bn");
assert.sameValue(
  0x4aabef2324cedff5387f1f65n | -0xbf2ed51ff75d380fd3be813ec6185780n, -0xbf2ed51fb554100cd330000ac600401bn,
  "0x4aabef2324cedff5387f1f65n | -0xbf2ed51ff75d380fd3be813ec6185780n === -0xbf2ed51fb554100cd330000ac600401bn");
assert.sameValue(
  -0xbf2ed51ff75d380fd3be813ec6185780n | -0x4aabef2324cedff5387f1f65n, -0x42092803008e813400181765n,
  "-0xbf2ed51ff75d380fd3be813ec6185780n | -0x4aabef2324cedff5387f1f65n === -0x42092803008e813400181765n");
assert.sameValue(
  -0x4aabef2324cedff5387f1f65n | -0xbf2ed51ff75d380fd3be813ec6185780n, -0x42092803008e813400181765n,
  "-0x4aabef2324cedff5387f1f65n | -0xbf2ed51ff75d380fd3be813ec6185780n === -0x42092803008e813400181765n");
assert.sameValue(-0xffffffffn | 0n, -0xffffffffn, "-0xffffffffn | 0n === -0xffffffffn");
assert.sameValue(0n | -0xffffffffn, -0xffffffffn, "0n | -0xffffffffn === -0xffffffffn");
assert.sameValue(
  -0xffffffffffffffffn | 0x10000000000000000n, -0xffffffffffffffffn,
  "-0xffffffffffffffffn | 0x10000000000000000n === -0xffffffffffffffffn");
assert.sameValue(
  0x10000000000000000n | -0xffffffffffffffffn, -0xffffffffffffffffn,
  "0x10000000000000000n | -0xffffffffffffffffn === -0xffffffffffffffffn");
assert.sameValue(
  -0xffffffffffffffffffffffffn | 0x10000000000000000n, -0xfffffffeffffffffffffffffn,
  "-0xffffffffffffffffffffffffn | 0x10000000000000000n === -0xfffffffeffffffffffffffffn");
assert.sameValue(
  0x10000000000000000n | -0xffffffffffffffffffffffffn, -0xfffffffeffffffffffffffffn,
  "0x10000000000000000n | -0xffffffffffffffffffffffffn === -0xfffffffeffffffffffffffffn");
