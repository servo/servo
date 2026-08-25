// Copyright (C) 2017 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: String should be parsed to BigInt according StringToBigInt
esid: sec-string-to-bigint
info: |
  ToBigInt ( argument )

  String:

  Let n be StringToBigInt(prim).
  If n is NaN, throw a SyntaxError exception.
  Return n.

  StringToBigInt ( argument )

  Replace the StrUnsignedDecimalLiteral production with DecimalDigits to not allow Infinity, decimal points, or exponents.

features: [BigInt]
---*/

assert.sameValue(BigInt("0b1111"), 15n);
assert.sameValue(BigInt("0b10"), 2n);
assert.sameValue(BigInt("0b0"), 0n);
assert.sameValue(BigInt("0b1"), 1n);

let binaryString = "0b1";
for (let i = 0; i < 128; i++)
  binaryString += "0";

assert.sameValue(BigInt(binaryString), 340282366920938463463374607431768211456n);

assert.sameValue(BigInt("0B1111"), 15n);
assert.sameValue(BigInt("0B10"), 2n);
assert.sameValue(BigInt("0B0"), 0n);
assert.sameValue(BigInt("0B1"), 1n);

binaryString = "0B1";
for (let i = 0; i < 128; i++)
  binaryString += "0";

assert.sameValue(BigInt(binaryString), 340282366920938463463374607431768211456n);
