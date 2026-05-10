// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isnan-number
description: >
  Return false if number is not NaN
info: |
  isNaN (number)

  1. Let num be ? ToNumber(number).
  2. If num is NaN, return true.
  3. Otherwise, return false.
---*/

assert.sameValue(isNaN(0), false, "0");
assert.sameValue(isNaN(-0), false, "-0");
assert.sameValue(isNaN(Math.pow(2, 53)), false, "Math.pow(2, 53)");
assert.sameValue(isNaN(-Math.pow(2, 53)), false, "-Math.pow(2, 53)");
assert.sameValue(isNaN(1), false, "1");
assert.sameValue(isNaN(-1), false, "-1");
assert.sameValue(isNaN(0.000001), false, "0.000001");
assert.sameValue(isNaN(-0.000001), false, "-0.000001");
assert.sameValue(isNaN(1e42), false, "1e42");
assert.sameValue(isNaN(-1e42), false, "-1e42");
assert.sameValue(isNaN(Infinity), false, "Infinity");
assert.sameValue(isNaN(-Infinity), false, "-Infinity");
