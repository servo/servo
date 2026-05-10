// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.10
description: >
    Object.is ( value1, value2 )

    ...
    6. If Type(x) is Number, then
      a. If x is NaN and y is NaN, return true.
      b. If x is +0 and y is -0, return false.
      c. If x is -0 and y is +0, return false.
      d. If x is the same Number value as y, return true.
      e. Return false.
    ...
---*/

assert.sameValue(Object.is(NaN, NaN), true, "`Object.is(NaN, NaN)` returns `true`");
assert.sameValue(Object.is(-0, -0), true, "`Object.is(-0, -0)` returns `true`");
assert.sameValue(Object.is(+0, +0), true, "`Object.is(+0, +0)` returns `true`");
assert.sameValue(Object.is(0, 0), true, "`Object.is(0, 0)` returns `true`");
