// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.log1p
description: >
  Return specific results
info: |
  Math.log1p ( x )

  If x is NaN, the result is NaN.
  If x is less than -1, the result is NaN.
  If x is -1, the result is -∞.
  If x is +0, the result is +0.
  If x is -0, the result is -0.
  If x is +∞, the result is +∞.
---*/

assert.sameValue(Math.log1p(NaN), NaN, "NaN");
assert.sameValue(Math.log1p(-1.000001), NaN, "-1.000001");
assert.sameValue(Math.log1p(-2), NaN, "-2");
assert.sameValue(Math.log1p(-Infinity), NaN, "-Infinity");
assert.sameValue(Math.log1p(-1), -Infinity, "-1");
assert.sameValue(Math.log1p(0), 0, "0");
assert.sameValue(Math.log1p(-0), -0, "-0");
assert.sameValue(Math.log1p(Infinity), Infinity, "Infinity");
