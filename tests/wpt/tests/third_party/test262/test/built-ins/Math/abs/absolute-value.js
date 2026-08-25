// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.abs
description: >
  Returns the absolute value of x
info: |
  Math.abs ( x )

  Returns the absolute value of x; the result has the same magnitude as x but
  has positive sign.
---*/

assert.sameValue(Math.abs(-42), 42, "-42");
assert.sameValue(Math.abs(42), 42, "42");
assert.sameValue(Math.abs(-0.000001), 0.000001, "-0.000001");
assert.sameValue(Math.abs(0.000001), 0.000001, "0.000001");
assert.sameValue(Math.abs(-1e-17), 1e-17, "-1e-17");
assert.sameValue(Math.abs(1e-17), 1e-17, "1e-17");
assert.sameValue(Math.abs(-9007199254740991), 9007199254740991, "-(2**53-1)");
assert.sameValue(Math.abs(9007199254740991), 9007199254740991, "2**53-1");
