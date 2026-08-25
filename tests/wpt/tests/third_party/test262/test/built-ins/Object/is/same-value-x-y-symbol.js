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

var a = Symbol();
var b = Symbol("description");

assert.sameValue(Object.is(a, a), true, "`Object.is(a, a)` returns `true`");
assert.sameValue(Object.is(b, b), true, "`Object.is(b, b)` returns `true`");
