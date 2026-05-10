// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.14
description: >
  Return abrupt result.
info: |
  26.1.14 Reflect.setPrototypeOf ( target, proto )

  ...
  3. Return target.[[SetPrototypeOf]](proto).
features: [Proxy, Reflect, Reflect.setPrototypeOf]
---*/

var target = {};
var p = new Proxy(target, {
  setPrototypeOf: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Reflect.setPrototypeOf(p, {});
});
