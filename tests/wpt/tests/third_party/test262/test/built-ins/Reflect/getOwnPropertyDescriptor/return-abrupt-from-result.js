// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.7
description: >
  Return abrupt result from getting the property descriptor.
info: |
  26.1.7 Reflect.getOwnPropertyDescriptor ( target, propertyKey )

  ...
  4. Let desc be target.[[GetOwnProperty]](key).
  5. ReturnIfAbrupt(desc).
  ...
features: [Proxy, Reflect]
---*/

var o1 = {};
var p = new Proxy(o1, {
  getOwnPropertyDescriptor: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Reflect.getOwnPropertyDescriptor(p, 'p1');
});
