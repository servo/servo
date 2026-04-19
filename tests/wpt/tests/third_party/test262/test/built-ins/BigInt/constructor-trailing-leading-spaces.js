// Copyright (C) 2017 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Trailing/Leading spaces should be ignored in BigInt constructor
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

assert.sameValue(BigInt("   0b1111"), 15n);
assert.sameValue(BigInt("18446744073709551616   "), 18446744073709551616n);
assert.sameValue(BigInt("   7   "), 7n);
assert.sameValue(BigInt("   -197   "), -197n);
assert.sameValue(BigInt("     "), 0n);
