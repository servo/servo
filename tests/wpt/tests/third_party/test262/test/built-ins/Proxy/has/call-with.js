// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.7
description: >
    A `with` variable check trigger trap.call(handler, target, P);
info: |
    [[HasProperty]] (P)

    ...
    9. Let booleanTrapResult be ToBoolean(Call(trap, handler, «target, P»)).
    ...
flags: [noStrict]
features: [Proxy]
---*/

var _handler, _target, _prop;
var target = {};
var handler = {
  has: function(t, prop) {
    _handler = this;
    _target = t;
    _prop = prop;

    return true;
  }
};
var p = new Proxy(target, handler);

with(p) {
  (attr);
}

assert.sameValue(_handler, handler);
assert.sameValue(_target, target);
assert.sameValue(_prop, "attr");
