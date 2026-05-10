// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.6
description: >
    Trap is called with handler as context and parameters are target, P, and the
    descriptor object.
info: |
    [[DefineOwnProperty]] (P, Desc)

    ...
    9. Let descObj be FromPropertyDescriptor(Desc).
    10. Let booleanTrapResult be ToBoolean(Call(trap, handler, «target, P,
    descObj»)).
    ...
features: [Proxy]
---*/

var _handler, _target, _prop, _desc;
var target = {};
var descriptor = {
  configurable: true,
  enumerable: true,
  writable: true,
  value: 1
};
var handler = {
  defineProperty: function(t, prop, desc) {
    _handler = this;
    _target = t;
    _prop = prop;
    _desc = desc;

    return true;
  }
};
var p = new Proxy(target, handler);

Object.defineProperty(p, "attr", descriptor);

assert.sameValue(_handler, handler);
assert.sameValue(_target, target);
assert.sameValue(_prop, "attr");

assert.sameValue(
  Object.keys(_desc).length, 4,
  "descriptor arg has the same amount of keys as given descriptor"
);

assert(_desc.configurable);
assert(_desc.writable);
assert(_desc.enumerable);
assert.sameValue(_desc.value, 1);
