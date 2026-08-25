// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.2.3.6
description: >
    Constructor is defined in the argument's prototype chain
info: |
    1. Let F be the this value.
    2. Return OrdinaryHasInstance(F, V).

    7.3.19 OrdinaryHasInstance (C, O)

    [...]
    7. Repeat
       a. Let O be O.[[GetPrototypeOf]]().
       b. ReturnIfAbrupt(O).
       c. If O is null, return false.
       d. If SameValue(P, O) is true, return true.
features: [Symbol.hasInstance]
---*/

var f = function() {};
var o = new f();
var o2 = Object.create(o);

assert.sameValue(f[Symbol.hasInstance](o), true);
assert.sameValue(f[Symbol.hasInstance](o2), true);
