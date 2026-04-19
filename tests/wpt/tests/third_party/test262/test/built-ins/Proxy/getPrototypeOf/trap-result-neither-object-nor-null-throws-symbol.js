// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.1
description: >
    Throw a TypeError exception if trap result is a Symbol.
features: [Proxy, Symbol]
---*/

var p = new Proxy({}, {
  getPrototypeOf: function() {
    return Symbol();
  }
});

assert.throws(TypeError, function() {
  Object.getPrototypeOf(p);
});
