// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.3
description: >
  Return abrupt result on defining a property.
info: |
  26.1.3 Reflect.defineProperty ( target, propertyKey, attributes )

  ...
  6. Return target.[[DefineOwnProperty]](key, desc).
  ...

  9.1.6.1 OrdinaryDefineOwnProperty (O, P, Desc)

  1. Let current be O.[[GetOwnProperty]](P).
  2. ReturnIfAbrupt(current).
  ...
features: [Proxy, Reflect]
---*/

var o = {};
var p = new Proxy(o, {
  defineProperty: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Reflect.defineProperty(p, 'p1', {});
});
