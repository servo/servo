// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: If the BigInt base and exponent are both 0n, return 1n
esid: sec-exp-operator-runtime-semantics-evaluation
info: |
  ExponentiationExpression: UpdateExpression ** ExponentiationExpression

  ...
  9. Return ? Type(base)::exponentiate(base, exponent).

  BigInt::exponentiate (base, exponent)

  1. If exponent < 0, throw a RangeError exception.
  2. If base is 0n and exponent is 0n, return 1n.
  3. Return a BigInt representing the mathematical value of base raised to the power exponent.
  ...
features: [BigInt, exponentiation]
---*/
assert.sameValue(0n ** 0n, 1n, 'The result of (0n ** 0n) is 1n');
