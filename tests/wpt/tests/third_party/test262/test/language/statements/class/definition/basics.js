// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class basics
---*/
var C = class C {}
assert.sameValue(typeof C, 'function', "`typeof C` is `'function'`");
assert.sameValue(
    Object.getPrototypeOf(C.prototype),
    Object.prototype,
    "`Object.getPrototypeOf(C.prototype)` returns `Object.prototype`"
);
assert.sameValue(
    Object.getPrototypeOf(C),
    Function.prototype,
    "`Object.getPrototypeOf(C)` returns `Function.prototype`"
);
assert.sameValue(C.name, 'C', "The value of `C.name` is `'C'`");

class D {}
assert.sameValue(typeof D, 'function', "`typeof D` is `'function'`");
assert.sameValue(
    Object.getPrototypeOf(D.prototype),
    Object.prototype,
    "`Object.getPrototypeOf(D.prototype)` returns `Object.prototype`"
);
assert.sameValue(
    Object.getPrototypeOf(D),
    Function.prototype,
    "`Object.getPrototypeOf(D)` returns `Function.prototype`"
);
assert.sameValue(D.name, 'D', "The value of `D.name` is `'D'`");

class D2 { constructor() {} }
assert.sameValue(D2.name, 'D2', "The value of `D2.name` is `'D2'`");

var E = class {}
assert.sameValue(E.name, 'E', "The value of `E.name` is `'E'`");

var F = class { constructor() {} };
assert.sameValue(F.name, 'F', "The value of `F.name` is `'F'`");
