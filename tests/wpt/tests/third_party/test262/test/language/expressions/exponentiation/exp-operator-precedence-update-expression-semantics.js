// Copyright (C) 2016 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Rick Waldron
esid: sec-update-expressions
description: Exponentiation Operator expression precedence of update operators
info: |
  ExponentiationExpression :
    ...
    UpdateExpression `**` ExponentiationExpression

  UpdateExpression :
    LeftHandSideExpression `++`
    LeftHandSideExpression `--`
    `++` UnaryExpression
    `--` UnaryExpression
features: [exponentiation]
---*/

var base = 4;
assert.sameValue(--base ** 2, 9, "(--base ** 2) === 9");
assert.sameValue(++base ** 2, 16, "(++base ** 2) === 16");
assert.sameValue(base++ ** 2, 16, "(base++ ** 2) === 16");
assert.sameValue(base-- ** 2, 25, "(base-- ** 2) === 25");

base = 4;

// --base ** --base ** 2 -> 3 ** 2 ** 2 -> 3 ** (2 ** 2) -> 81
assert.sameValue(
  --base ** --base ** 2,
  Math.pow(3, Math.pow(2, 2)),
  "(--base ** --base ** 2) === Math.pow(3, Math.pow(2, 2))"
);

// ++base ** ++base ** 2 -> 3 ** 4 ** 2 -> 3 ** (4 ** 2) -> 43046721
assert.sameValue(
  ++base ** ++base ** 2,
  Math.pow(3, Math.pow(4, 2)),
  "(++base ** ++base ** 2) === Math.pow(3, Math.pow(4, 2))"
);

base = 4;

// base-- ** base-- ** 2 -> 4 ** 3 ** 2 -> 4 ** (3 ** 2) -> 262144
assert.sameValue(
  base-- ** base-- ** 2,
  Math.pow(4, Math.pow(3, 2)),
  "(base-- ** base-- ** 2) === Math.pow(4, Math.pow(3, 2))"
);

// base++ ** base++ ** 2 -> 2 ** 3 ** 2 -> 2 ** (3 ** 2) -> 262144
assert.sameValue(
  base++ ** base++ ** 2,
  Math.pow(2, Math.pow(3, 2)),
  "(base++ ** base++ ** 2) === Math.pow(2, Math.pow(3, 2))"
);

