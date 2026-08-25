// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.6
description: >
  Return abrupt result from get a property value.
info: |
  26.1.6 Reflect.get ( target, propertyKey [ , receiver ])

  ...
  5. Return target.[[Get]](key, receiver).
features: [Reflect]
---*/

var o1 = {};
Object.defineProperty(o1, 'p1', {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Reflect.get(o1, 'p1');
});

// Abrupt from the prototype property
var o2 = Object.create(o1);
assert.throws(Test262Error, function() {
  Reflect.get(o2, 'p1');
});
