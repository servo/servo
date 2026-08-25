// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.10
description: >
    Object.is ( value1, value2 )

    ...
    6. If Type(x) is Symbol, then
      a. If x and y are both the same Symbol value, 
          return true; otherwise, return false.
    ...
features: [Symbol]
---*/

assert.sameValue(
  Object.is(Symbol(), Symbol()),
  false,
  "`Object.is(Symbol(), Symbol())` returns `false`"
);
assert.sameValue(
  Object.is(Symbol('description'), Symbol('description')),
  false,
  "`Object.is(Symbol('description'), Symbol('description'))` returns `false`"
);
