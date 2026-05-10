// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
includes: [sm/non262-Math-shell.js]
---*/
// If x is NaN, the result is NaN.
assert.sameValue(Math.trunc(NaN), NaN);

// If x is −0, the result is −0.
assert.sameValue(Math.trunc(-0), -0);

// If x is +0, the result is +0.
assert.sameValue(Math.trunc(+0), +0);

// If x is +∞, the result is +∞.
assert.sameValue(Math.trunc(Infinity), Infinity);

// If x is −∞, the result is −∞.
assert.sameValue(Math.trunc(-Infinity), -Infinity);

// Other boundary cases.
var MAX_NONINTEGER_VALUE       = 4503599627370495.5;
var TRUNC_MAX_NONINTEGER_VALUE = 4503599627370495;

assert.sameValue(Math.trunc(Number.MIN_VALUE), +0);
assert.sameValue(Math.trunc(ONE_MINUS_EPSILON), +0);
assert.sameValue(Math.trunc(ONE_PLUS_EPSILON), 1);
assert.sameValue(Math.trunc(MAX_NONINTEGER_VALUE), TRUNC_MAX_NONINTEGER_VALUE);
assert.sameValue(Math.trunc(Number.MAX_VALUE), Number.MAX_VALUE);

assert.sameValue(Math.trunc(-Number.MIN_VALUE), -0);
assert.sameValue(Math.trunc(-ONE_MINUS_EPSILON), -0);
assert.sameValue(Math.trunc(-ONE_PLUS_EPSILON), -1);
assert.sameValue(Math.trunc(-MAX_NONINTEGER_VALUE), -TRUNC_MAX_NONINTEGER_VALUE);
assert.sameValue(Math.trunc(-Number.MAX_VALUE), -Number.MAX_VALUE);

// Other cases.
for (var i = 1, f = 1.1; i < 20; i++, f += 1.0)
    assert.sameValue(Math.trunc(f), i);

for (var i = -1, f = -1.1; i > -20; i--, f -= 1.0)
    assert.sameValue(Math.trunc(f), i);

assert.sameValue(Math.trunc(1e40 + 0.5), 1e40);

assert.sameValue(Math.trunc(1e300), 1e300);
assert.sameValue(Math.trunc(-1e300), -1e300);
assert.sameValue(Math.trunc(1e-300), 0);
assert.sameValue(Math.trunc(-1e-300), -0);

assert.sameValue(Math.trunc(+0.9999), +0);
assert.sameValue(Math.trunc(-0.9999), -0);


