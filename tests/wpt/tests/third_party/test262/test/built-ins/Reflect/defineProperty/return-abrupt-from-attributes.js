// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.3
description: >
  Return abrupt from ToPropertyDescriptor(attributes).
info: |
  26.1.3 Reflect.defineProperty ( target, propertyKey, attributes )

  ...
  4. Let desc be ToPropertyDescriptor(attributes).
  5. ReturnIfAbrupt(desc).
  ...
features: [Reflect]
---*/

var attributes = {};

Object.defineProperty(attributes, 'enumerable', {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Reflect.defineProperty({}, 'a', attributes);
});
