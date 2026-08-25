// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.7
description: >
  Throws a TypeError if target is a Symbol
info: |
  26.1.7 Reflect.getOwnPropertyDescriptor ( target, propertyKey )

  1. If Type(target) is not Object, throw a TypeError exception.
  ...
features: [Reflect, Symbol]
---*/

assert.throws(TypeError, function() {
  Reflect.getOwnPropertyDescriptor(Symbol(1), 'p');
});
