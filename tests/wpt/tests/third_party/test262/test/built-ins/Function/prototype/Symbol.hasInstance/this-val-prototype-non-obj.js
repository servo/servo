// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.2.3.6
description: Error thrown when accessing `prototype` property of `this` value
info: |
    1. Let F be the this value.
    2. Return OrdinaryHasInstance(F, V).

    7.3.19 OrdinaryHasInstance (C, O)

    [...]
    4. Let P be Get(C, "prototype").
    5. ReturnIfAbrupt(P).
    6. If Type(P) is not Object, throw a TypeError exception.
features: [Symbol, Symbol.hasInstance]
---*/

var f = function() {};

f.prototype = undefined;
assert.throws(TypeError, function() {
  f[Symbol.hasInstance]({});
});

f.prototype = null;
assert.throws(TypeError, function() {
  f[Symbol.hasInstance]({});
});

f.prototype = true;
assert.throws(TypeError, function() {
  f[Symbol.hasInstance]({});
});

f.prototype = 'string';
assert.throws(TypeError, function() {
  f[Symbol.hasInstance]({});
});

f.prototype = Symbol();
assert.throws(TypeError, function() {
  f[Symbol.hasInstance]({});
});

f.prototype = 86;
assert.throws(TypeError, function() {
  f[Symbol.hasInstance]({});
});
