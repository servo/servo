// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Properties of Math.exp that are guaranteed by the spec.

// If x is NaN, the result is NaN.
assert.sameValue(Math.exp(NaN), NaN);

// If x is +0, the result is 1.
assert.sameValue(Math.exp(+0), 1);

// If x is −0, the result is 1.
assert.sameValue(Math.exp(-0), 1);

// If x is +∞, the result is +∞.
assert.sameValue(Math.exp(Infinity), Infinity);

// If x is −∞, the result is +0.
assert.sameValue(Math.exp(-Infinity), +0);


// Not guaranteed by the specification, but generally assumed to hold.

// If x is 1, the result is Math.E.
assert.sameValue(Math.exp(1), Math.E);

// If x is -1, the result is 1/Math.E.
assert.sameValue(Math.exp(-1), 1 / Math.E);


