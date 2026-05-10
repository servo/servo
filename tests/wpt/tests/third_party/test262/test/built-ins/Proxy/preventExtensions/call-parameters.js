// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.4
description: >
    Trap is called with handler on its context and target as the first
    parameter.
info: |
    [[PreventExtensions]] ( )

    ...
    8. Let booleanTrapResult be ToBoolean(Call(trap, handler, «target»)).
    ...
features: [Proxy]
---*/

var _target, _handler;
var target = {};
var handler = {
  preventExtensions: function(t) {
    _handler = this;
    _target = t;

    return Object.preventExtensions(target);
  }
};
var p = new Proxy(target, handler);

Object.preventExtensions(p);

assert.sameValue(_handler, handler);
assert.sameValue(_target, target);
