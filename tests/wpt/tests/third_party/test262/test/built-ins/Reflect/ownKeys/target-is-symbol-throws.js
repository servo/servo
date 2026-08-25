// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.11
description: >
  Throws a TypeError if target is a Symbol
info: |
  26.1.11 Reflect.ownKeys ( target )

  1. If Type(target) is not Object, throw a TypeError exception.
  ...
features: [Reflect, Symbol]
---*/

assert.throws(TypeError, function() {
  Reflect.ownKeys(Symbol(1));
});
