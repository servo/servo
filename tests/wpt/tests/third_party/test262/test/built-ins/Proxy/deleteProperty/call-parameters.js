// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.10
description: >
    [[Delete]] (P)

    9. Let booleanTrapResult be ToBoolean(Call(trap, handler, «target, P»)).
info: |
    6.1.7.2 Object Internal Methods and Internal Slots

    (...) Receiver is used as the this value when evaluating the code
features: [Proxy]
---*/

var _handler, _target, _prop;
var target = {
  attr: 1
};
var handler = {
  deleteProperty: function(t, prop) {
    _handler = this;
    _target = t;
    _prop = prop;
    return delete t[prop];
  }
};
var p = new Proxy(target, handler);

delete p.attr;

assert.sameValue(_handler, handler, "handler object as the trap context");
assert.sameValue(_target, target, "first argument is the target object");
assert.sameValue(_prop, "attr", "second argument is the property name");
