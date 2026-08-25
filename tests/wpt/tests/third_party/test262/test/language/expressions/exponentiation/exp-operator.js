// Copyright (C) 2016 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Rick Waldron
esid: sec-exp-operator
description: >
    Performs exponential calculation on operands. Same algorithm as %MathPow%(base, exponent)
features: [exponentiation]
---*/

var exponent = 2;
assert.sameValue(2 ** 3, 8, "(2 ** 3) === 8");
assert.sameValue(3 * 2 ** 3, 24, "(3 * 2 ** 3) === 24");
assert.sameValue(2 ** ++exponent, 8, "(2 ** ++exponent) === 8");
assert.sameValue(2 ** -1 * 2, 1, "(2 ** -1 * 2) === 1");
assert.sameValue(2 ** 2 * 4, 16, "(2 ** 2 * 4) === 16");
assert.sameValue(2 ** 2 / 2, 2, "(2 ** 2 / 2) === 2");
assert.sameValue(2 ** (3 ** 2), 512, "(2 ** (3 ** 2)) === 512");
assert.sameValue(2 ** 3 ** 2, 512, "(2 ** 3 ** 2) === 512");
assert.sameValue(16 / 2 ** 2, 4, "(16 / 2 ** 2) === 4");
