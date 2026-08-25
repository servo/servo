// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.6
description: >
    Trap returns a boolean. Checking on false values.
info: |
    [[DefineOwnProperty]] (P, Desc)

    ...
    12. If booleanTrapResult is false, return false.
    ...
features: [Proxy, Reflect]
---*/

var target = {};
var p = new Proxy(target, {
  defineProperty: function(t, prop, desc) {
    return 0;
  }
});

assert.sameValue(Reflect.defineProperty(p, "attr", {}), false);
assert.sameValue(
  Object.getOwnPropertyDescriptor(target, "attr"),
  undefined
);
