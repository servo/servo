// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class side effect in extends
---*/
var calls = 0;
class C {}
class D extends (calls++, C) {}
assert.sameValue(calls, 1, "The value of `calls` is `1`");
assert.sameValue(typeof D, 'function', "`typeof D` is `'function'`");
assert.sameValue(Object.getPrototypeOf(D), C, "`Object.getPrototypeOf(D)` returns `C`");
assert.sameValue(
    C.prototype,
    Object.getPrototypeOf(D.prototype),
    "The value of `C.prototype` is `Object.getPrototypeOf(D.prototype)`"
);
