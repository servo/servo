// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.9
description: >
  Throws a TypeError if target is not an Object.
info: |
  26.1.9 Reflect.has ( target, propertyKey )

  1. If Type(target) is not Object, throw a TypeError exception.
  ...
features: [Reflect]
---*/

assert.throws(TypeError, function() {
  Reflect.has(1, 'p');
});

assert.throws(TypeError, function() {
  Reflect.has(null, 'p');
});

assert.throws(TypeError, function() {
  Reflect.has(undefined, 'p');
});

assert.throws(TypeError, function() {
  Reflect.has('', 'p');
});
