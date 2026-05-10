// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Properties of Math.asinh that are guaranteed by the spec.

// If x is NaN, the result is NaN.
assert.sameValue(Math.asinh(NaN), NaN);

// If x is +0, the result is +0.
assert.sameValue(Math.asinh(+0), +0);

// If x is −0, the result is −0.
assert.sameValue(Math.asinh(-0), -0);

// If x is +∞, the result is +∞.
assert.sameValue(Math.asinh(Infinity), Infinity);

// If x is −∞, the result is −∞.
assert.sameValue(Math.asinh(-Infinity), -Infinity);


