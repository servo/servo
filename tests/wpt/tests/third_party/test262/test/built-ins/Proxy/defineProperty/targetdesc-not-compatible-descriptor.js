// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.6
description: >
    Throw a TypeError exception if Desc and target property descriptor are not
    compatible and trap result is true.
info: |
    [[DefineOwnProperty]] (P, Desc)

    ...
    20. Else targetDesc is not undefined,
        a. If IsCompatiblePropertyDescriptor(extensibleTarget, Desc ,
        targetDesc) is false, throw a TypeError exception.
    ...
features: [Proxy]
---*/

var target = {};
var p = new Proxy(target, {
  defineProperty: function(t, prop, desc) {
    return true;
  }
});

Object.defineProperty(target, "foo", {
  value: 1
});

assert.throws(TypeError, function() {
  Object.defineProperty(p, "foo", {
    value: 2
  });
});
