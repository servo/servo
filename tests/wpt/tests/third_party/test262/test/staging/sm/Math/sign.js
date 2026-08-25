// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// If x is NaN, the result is NaN.
assert.sameValue(Math.sign(NaN), NaN);

// If x is −0, the result is −0.
assert.sameValue(Math.sign(-0), -0);

// If x is +0, the result is +0.
assert.sameValue(Math.sign(+0), +0);

// If x is negative and not −0, the result is −1.
assert.sameValue(Math.sign(-Number.MIN_VALUE), -1);
assert.sameValue(Math.sign(-Number.MAX_VALUE), -1);
assert.sameValue(Math.sign(-Infinity), -1);

for (var i = -1; i > -20; i--)
    assert.sameValue(Math.sign(i), -1);

assert.sameValue(Math.sign(-1e-300), -1);
assert.sameValue(Math.sign(-0x80000000), -1);

// If x is positive and not +0, the result is +1.
assert.sameValue(Math.sign(Number.MIN_VALUE), +1);
assert.sameValue(Math.sign(Number.MAX_VALUE), +1);
assert.sameValue(Math.sign(Infinity), +1);

for (var i = 1; i < 20; i++)
    assert.sameValue(Math.sign(i), +1);

assert.sameValue(Math.sign(+1e-300), +1);
assert.sameValue(Math.sign(0x80000000), +1);
assert.sameValue(Math.sign(0xffffffff), +1);


