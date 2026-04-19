// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.isfinite
description: >
  Return true for valid finite numbers
info: |
  Number.isFinite ( number )

  1. If Type(number) is not Number, return false.
  2. If number is NaN, +∞, or -∞, return false.
  3. Otherwise, return true.
---*/

assert.sameValue(Number.isFinite(-10), true, "-10");
assert.sameValue(Number.isFinite(-0), true, "-0");
assert.sameValue(Number.isFinite(0), true, "0");
assert.sameValue(Number.isFinite(10), true, "10");
assert.sameValue(Number.isFinite(1e10), true, "1e10");
assert.sameValue(Number.isFinite(10.10), true, "10.10");
assert.sameValue(Number.isFinite(9007199254740991), true, "9007199254740991");
assert.sameValue(Number.isFinite(-9007199254740991), true, "-9007199254740991");
assert.sameValue(Number.isFinite(Number.MAX_VALUE), true, "Number.MAX_VALUE");
