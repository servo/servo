// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
includes: [sm/non262-Math-shell.js]
---*/
// Properties of Math.acosh that are guaranteed by the spec.

// If x is NaN, the result is NaN.
assert.sameValue(Math.acosh(NaN), NaN);

// If x is less than 1, the result is NaN.
assert.sameValue(Math.acosh(ONE_MINUS_EPSILON), NaN);
assert.sameValue(Math.acosh(Number.MIN_VALUE), NaN);
assert.sameValue(Math.acosh(+0), NaN);
assert.sameValue(Math.acosh(-0), NaN);
assert.sameValue(Math.acosh(-Number.MIN_VALUE), NaN);
assert.sameValue(Math.acosh(-1), NaN);
assert.sameValue(Math.acosh(-Number.MAX_VALUE), NaN);
assert.sameValue(Math.acosh(-Infinity), NaN);

for (var i = -20; i < 1; i++)
    assert.sameValue(Math.acosh(i), NaN);

// If x is 1, the result is +0.
assert.sameValue(Math.acosh(1), +0);

// If x is +∞, the result is +∞.
assert.sameValue(Math.acosh(Number.POSITIVE_INFINITY), Number.POSITIVE_INFINITY);


