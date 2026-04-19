// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.4
description: >
  Return abrupt result from deleting a property.
info: |
  26.1.4 Reflect.deleteProperty ( target, propertyKey )

  ...
  6. Return target.[[DefineOwnProperty]](key, desc).
  ...
features: [Proxy, Reflect]
---*/

var o = {};
var p = new Proxy(o, {
  deleteProperty: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Reflect.deleteProperty(p, 'p1');
});
