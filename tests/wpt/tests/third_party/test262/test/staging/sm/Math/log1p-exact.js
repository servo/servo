// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
includes: [sm/non262-Math-shell.js]
---*/
// Properties of Math.log1p that are guaranteed by the spec.

// If x is NaN, the result is NaN.
assert.sameValue(Math.log1p(NaN), NaN);

// If x is less than -1, the result is NaN.
assert.sameValue(Math.log1p(-1 - 1e-10), NaN);
assert.sameValue(Math.log1p(-1 - 1e-5), NaN);
assert.sameValue(Math.log1p(-1 - 1e-1), NaN);
assert.sameValue(Math.log1p(-ONE_PLUS_EPSILON), NaN);

for (var i = -2; i > -20; i--)
    assert.sameValue(Math.log1p(i), NaN);

// If x is -1, the result is -∞.
assert.sameValue(Math.log1p(-1), -Infinity);

// If x is +0, the result is +0.
assert.sameValue(Math.log1p(+0), +0);

// If x is −0, the result is −0.
assert.sameValue(Math.log1p(-0), -0);

// If x is +∞, the result is +∞.
assert.sameValue(Math.log1p(Infinity), Infinity);


