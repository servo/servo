// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Properties of Math.tanh that are guaranteed by the spec.

// If x is NaN, the result is NaN.
assert.sameValue(Math.tanh(NaN), NaN);

// If x is +0, the result is +0.
assert.sameValue(Math.tanh(+0), +0);

// If x is −0, the result is −0.
assert.sameValue(Math.tanh(-0), -0);

// If x is +∞, the result is +1.
assert.sameValue(Math.tanh(Number.POSITIVE_INFINITY), +1);

// If x is −∞, the result is -1.
assert.sameValue(Math.tanh(Number.NEGATIVE_INFINITY), -1);


