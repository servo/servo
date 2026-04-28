// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.2
description: >
  Throws a TypeError if `newTarget` is not a constructor.
info: |
  26.1.2 Reflect.construct ( target, argumentsList [, newTarget] )

  ...
  2. If newTarget is not present, let newTarget be target.
  3. Else, if IsConstructor(newTarget) is false, throw a TypeError exception.
  ...
features: [Reflect, Reflect.construct]
---*/

assert.throws(TypeError, function() {
  Reflect.construct(function() {}, [], 1);
});

assert.throws(TypeError, function() {
  Reflect.construct(function() {}, [], null);
});

assert.throws(TypeError, function() {
  Reflect.construct(function() {}, [], {});
});

assert.throws(TypeError, function() {
  Reflect.construct(function() {}, [], Date.now);
});
