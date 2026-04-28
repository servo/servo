// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.5
description: >
    Trap is called with hander context and parameters are target and P
info: |
    [[GetOwnProperty]] (P)

    ...
    9. Let trapResultObj be Call(trap, handler, «target, P»).
    ...
features: [Proxy]
---*/

var _target, _handler, _prop;
var target = {
  attr: 1
};
var handler = {
  getOwnPropertyDescriptor: function(t, prop) {
    _target = t;
    _handler = this;
    _prop = prop;

    return Object.getOwnPropertyDescriptor(t, prop);
  }
};
var p = new Proxy(target, handler);

Object.getOwnPropertyDescriptor(p, "attr");

assert.sameValue(_handler, handler);
assert.sameValue(_target, target);
assert.sameValue(_prop, "attr");
