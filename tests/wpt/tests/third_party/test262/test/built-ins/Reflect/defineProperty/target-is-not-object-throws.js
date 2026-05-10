// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.3
description: >
  Throws a TypeError if target is not an Object.
info: |
  26.1.3 Reflect.defineProperty ( target, propertyKey, attributes )

  1. If Type(target) is not Object, throw a TypeError exception.
  ...
features: [Reflect]
---*/

assert.throws(TypeError, function() {
  Reflect.defineProperty(1, 'p', {});
});

assert.throws(TypeError, function() {
  Reflect.defineProperty(null, 'p', {});
});

assert.throws(TypeError, function() {
  Reflect.defineProperty(undefined, 'p', {});
});

assert.throws(TypeError, function() {
  Reflect.defineProperty('', 'p', {});
});
