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
features: [Symbol.hasInstance]
---*/

// Create a callable object without a `prototype` property
var f = Object.getOwnPropertyDescriptor({
  get f() {}
}, 'f').get;

Object.defineProperty(f, 'prototype', {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  f[Symbol.hasInstance]({});
});
