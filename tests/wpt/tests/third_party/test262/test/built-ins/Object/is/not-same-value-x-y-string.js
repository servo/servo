// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.10
description: >
    Object.is ( value1, value2 )

    7.2.9 SameValue(x, y)

    ...
    7. If Type(x) is String, then
      a. If x and y are exactly the same sequence of code units 
        (same length and same code units at corresponding indices) 
        return true; otherwise, return false.
    ...
---*/

assert.sameValue(Object.is('', true), false, "`Object.is('', true)` returns `false`");
assert.sameValue(Object.is('', 0), false, "`Object.is('', 0)` returns `false`");
assert.sameValue(Object.is('', {}), false, "`Object.is('', {})` returns `false`");
assert.sameValue(
  Object.is('', undefined),
  false,
  "`Object.is('', undefined)` returns `false`"
);
assert.sameValue(Object.is('', null), false, "`Object.is('', null)` returns `false`");
assert.sameValue(Object.is('', NaN), false, "`Object.is('', NaN)` returns `false`");
