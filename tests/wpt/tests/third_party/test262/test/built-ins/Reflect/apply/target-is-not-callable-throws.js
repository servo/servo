// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.1
description: >
  Throws a TypeError if `target` is not callable.
info: |
  26.1.1 Reflect.apply ( target, thisArgument, argumentsList )

  1. If IsCallable(target) is false, throw a TypeError exception.
  ...

  7.2.3 IsCallable ( argument )

  1. ReturnIfAbrupt(argument).
  2. If Type(argument) is not Object, return false.
  3. If argument has a [[Call]] internal method, return true.
  4. Return false.
features: [Reflect]
---*/

assert.throws(TypeError, function() {
  Reflect.apply(1, 1, []);
});

assert.throws(TypeError, function() {
  Reflect.apply(null, 1, []);
});

assert.throws(TypeError, function() {
  Reflect.apply({}, 1, []);
});
