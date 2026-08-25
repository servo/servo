// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Properties of Math.log10 that are guaranteed by the spec.

// If x is NaN, the result is NaN.
assert.sameValue(Math.log10(NaN), NaN);

// If x is less than 0, the result is NaN.
assert.sameValue(Math.log10(-1e-10), NaN);
assert.sameValue(Math.log10(-1e-5), NaN);
assert.sameValue(Math.log10(-1e-1), NaN);
assert.sameValue(Math.log10(-Number.MIN_VALUE), NaN);
assert.sameValue(Math.log10(-Number.MAX_VALUE), NaN);
assert.sameValue(Math.log10(-Infinity), NaN);

for (var i = -1; i > -10; i--)
    assert.sameValue(Math.log10(i), NaN);

// If x is +0, the result is −∞.
assert.sameValue(Math.log10(+0), -Infinity);

// If x is −0, the result is −∞.
assert.sameValue(Math.log10(-0), -Infinity);

// If x is 1, the result is +0.
assert.sameValue(Math.log10(1), +0);

// If x is +∞, the result is +∞.
assert.sameValue(Math.log10(Infinity), Infinity);


