// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isfinite-number
description: >
  Return true if number is not NaN, Infinity or -Infinity
info: |
  isFinite (number)

  1. Let num be ? ToNumber(number).
  2. If num is NaN, +∞, or -∞, return false.
  3. Otherwise, return true.
---*/

assert.sameValue(isFinite(0), true, "0");
assert.sameValue(isFinite(-0), true, "-0");
assert.sameValue(isFinite(Math.pow(2, 53)), true, "Math.pow(2, 53)");
assert.sameValue(isFinite(-Math.pow(2, 53)), true, "-Math.pow(2, 53)");
assert.sameValue(isFinite(1), true, "1");
assert.sameValue(isFinite(-1), true, "-1");
assert.sameValue(isFinite(0.000001), true, "0.000001");
assert.sameValue(isFinite(-0.000001), true, "-0.000001");
assert.sameValue(isFinite(1e42), true, "1e42");
assert.sameValue(isFinite(-1e42), true, "-1e42");
