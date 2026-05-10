// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.2
description: >
    Trap is called with handler on its context, first parameter is target and
    second parameter is the given value.
info: |
    [[SetPrototypeOf]] (V)

    ...
    9. Let booleanTrapResult be ToBoolean(Call(trap, handler, «target, V»)).
    ...
features: [Proxy]
---*/

var _handler, _target, _value;
var target = {};
var val = {
  foo: 1
};
var handler = {
  setPrototypeOf: function(t, v) {
    _handler = this;
    _target = t;
    _value = v;

    Object.setPrototypeOf(t, v);

    return true;
  }
};
var p = new Proxy(target, handler);

Object.setPrototypeOf(p, val);

assert.sameValue(_handler, handler);
assert.sameValue(_target, target);
assert.sameValue(_value, val);
