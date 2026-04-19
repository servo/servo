// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class implicit constructor
---*/
class C {}
var c = new C();
assert.sameValue(
    Object.getPrototypeOf(c),
    C.prototype,
    "`Object.getPrototypeOf(c)` returns `C.prototype`"
);
