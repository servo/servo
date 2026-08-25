// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: BigInt.asUintN type coercion for bigint parameter
esid: sec-bigint.asuintn
info: |
  BigInt.asUintN ( bits, bigint )

  2. Let bigint ? ToBigInt(bigint).
features: [BigInt]
---*/

assert.sameValue(BigInt.asUintN(2, 0n), 0n);
assert.sameValue(BigInt.asUintN(2, -0n), 0n);
assert.sameValue(BigInt.asUintN(2, false), 0n, "ToBigInt: false => 0n");
assert.sameValue(BigInt.asUintN(2, true), 1n, "ToBigInt: true => 1n");
assert.sameValue(BigInt.asUintN(2, "1"), 1n, "ToBigInt: parse BigInt");
assert.sameValue(BigInt.asUintN(2, "-0"), 0n, "ToBigInt: parse BigInt");
assert.sameValue(BigInt.asUintN(2, ""), 0n, "ToBigInt: empty String => 0n");
assert.sameValue(BigInt.asUintN(2, "     "), 0n, "ToBigInt: String with only whitespace => 0n");
assert.sameValue(BigInt.asUintN(2, []), 0n, "ToBigInt: .toString() => empty String => 0n");
assert.sameValue(BigInt.asUintN(2, [1]), 1n, "ToBigInt: .toString() => parse BigInt");
assert.sameValue(BigInt.asUintN(3, 10n), 2n);
assert.sameValue(BigInt.asUintN(3, "10"), 2n, "ToBigInt: parse BigInt");
assert.sameValue(BigInt.asUintN(3, "0b1010"), 2n, "ToBigInt: parse BigInt binary");
assert.sameValue(BigInt.asUintN(3, "0o12"), 2n, "ToBigInt: parse BigInt octal");
assert.sameValue(BigInt.asUintN(3, "0xa"), 2n, "ToBigInt: parse BigInt hex");
assert.sameValue(BigInt.asUintN(3, "    0xa    "), 2n,
  "ToBigInt: parse BigInt ignore leading/trailing whitespace");
assert.sameValue(BigInt.asUintN(3, "     10     "), 2n,
  "ToBigInt: parse BigInt ignore leading/trailing whitespace");
assert.sameValue(BigInt.asUintN(3, [10n]), 2n, "ToBigInt: .toString() => parse BigInt");
assert.sameValue(BigInt.asUintN(3, ["10"]), 2n, "ToBigInt: .toString() => parse BigInt");
assert.sameValue(BigInt.asUintN(4, 12345678901234567890003n), 3n);
assert.sameValue(BigInt.asUintN(4, "12345678901234567890003"), 3n, "ToBigInt: parse BigInt");
assert.sameValue(BigInt.asUintN(4,
    "0b10100111010100001010110110010011100111011001110001010000100100010001010011"), 3n,
  "ToBigInt: parse BigInt binary");
assert.sameValue(BigInt.asUintN(4, "0o2472412662347316120442123"), 3n,
  "ToBigInt: parse BigInt octal");
assert.sameValue(BigInt.asUintN(4, "0x29d42b64e7671424453"), 3n, "ToBigInt: parse BigInt hex");
assert.sameValue(BigInt.asUintN(4, "    0x29d42b64e7671424453    "), 3n,
  "ToBigInt: parse BigInt ignore leading/trailing whitespace");
assert.sameValue(BigInt.asUintN(4, "     12345678901234567890003     "), 3n,
  "ToBigInt: parse BigInt ignore leading/trailing whitespace");
assert.sameValue(BigInt.asUintN(4, [12345678901234567890003n]), 3n,
  "ToBigInt: .toString() => parse BigInt");
assert.sameValue(BigInt.asUintN(4, ["12345678901234567890003"]), 3n,
  "ToBigInt: .toString() => parse BigInt");
