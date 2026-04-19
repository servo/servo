// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.2.3.6
description: >
    Error thrown when invoking argument's [[GetPrototypeOf]] internal method
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
features: [Proxy, Symbol.hasInstance]
---*/

var o = new Proxy({}, {
  getPrototypeOf: function() {
    throw new Test262Error();
  }
});
var o2 = Object.create(o);
var f = function() {};

assert.throws(Test262Error, function() {
  f[Symbol.hasInstance](o);
});

assert.throws(Test262Error, function() {
  f[Symbol.hasInstance](o2);
});
