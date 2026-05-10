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

assert.sameValue(Object.is('', ''), true, "`Object.is('', '')` returns `true`");
assert.sameValue(
  Object.is('foo', 'foo'),
  true,
  "`Object.is('foo', 'foo')` returns `true`"
);
assert.sameValue(
  Object.is(String('foo'), String('foo')),
  true,
  "`Object.is(String('foo'), String('foo'))` returns `true`"
);
