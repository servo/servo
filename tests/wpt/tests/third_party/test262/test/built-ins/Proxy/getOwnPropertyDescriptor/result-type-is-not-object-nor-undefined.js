// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.5
description: >
    Throws a TypeError exception if trap result is neither Object nor Undefined
info: |
    [[GetOwnProperty]] (P)

    ...
    11. If Type(trapResultObj) is neither Object nor Undefined, throw a
    TypeError exception.
    ...
features: [Proxy, Symbol]
---*/

var target = {
  number: 1,
  symbol: Symbol(),
  string: '',
  boolean: true
};
var p = new Proxy(target, {
  getOwnPropertyDescriptor: function(t, prop) {
    return t[prop];
  }
});

assert.throws(TypeError, function() {
  Object.getOwnPropertyDescriptor(p, "number");
});

assert.throws(TypeError, function() {
  Object.getOwnPropertyDescriptor(p, "string");
});

assert.throws(TypeError, function() {
  Object.getOwnPropertyDescriptor(p, "symbol");
});

assert.throws(TypeError, function() {
  Object.getOwnPropertyDescriptor(p, "boolean");
});
