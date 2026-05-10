// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.2
description: >
  Throws a TypeError if `target` is not a constructor.
info: |
  26.1.2 Reflect.construct ( target, argumentsList [, newTarget] )

  1. If IsConstructor(target) is false, throw a TypeError exception.
features: [Reflect, Reflect.construct]
---*/

assert.throws(TypeError, function() {
  Reflect.construct(1, []);
});

assert.throws(TypeError, function() {
  Reflect.construct(null, []);
});

assert.throws(TypeError, function() {
  Reflect.construct({}, []);
});

assert.throws(TypeError, function() {
  Reflect.construct(Date.now, []);
});
